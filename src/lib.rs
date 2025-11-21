use log::{debug, error, info, trace, warn};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use chrono::Utc;
use http::header::HeaderValue;
use serde_json::json;
use tokio::{
    sync::{broadcast, oneshot, Mutex as TokioMutex},
    time::timeout,
};
use yawc::{CompressionLevel, HttpRequestBuilder, Options, WebSocket};
use rustls::crypto::ring;

pub mod api;
pub mod constants;
pub mod errors;
pub mod models;
pub mod navigation;

pub mod transport;

use transport::{
    TransportFactory, 
    TransportReader, 
    TransportWriter,
    web::WebTransport,
    mobile::MobileTransport
};

use constants::Constants;
use errors::{ClientResult, Error};
use models::{Request, Response};

struct ClientState {
    writer: Option<Box<dyn TransportWriter>>,
    seq: u64,
    temp_token: Option<String>,
    token: Option<String>,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<ClientResult<Response>>>>>,
    shutdown_tx: Option<broadcast::Sender<()>>,
    user_id: Option<u64>,
    action_id: u64,
    session_id: i64,
    device_id: Option<String>,
    mt_instance: Option<String>,
    current_screen: String,
}

pub enum ClientMode {
    Web,
    Mobile,
}

#[derive(Clone)]
pub struct MaxClient {
    state: Arc<TokioMutex<ClientState>>,
    event_tx: broadcast::Sender<Response>,
}

impl MaxClient {
    pub fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        let (event_tx, _) = broadcast::channel(20);
        MaxClient {
            state: Arc::new(TokioMutex::new(ClientState {
                writer: None,
                seq: 0,
                temp_token: None,
                token: None,
                pending: Arc::new(Mutex::new(HashMap::new())),
                shutdown_tx: Some(shutdown_tx),
                user_id: None,
                action_id: 0,
                session_id: Utc::now().timestamp_millis(),
                device_id: None,
                mt_instance: None, // TODO IDK что это и зачем
                current_screen: "chats_list_tab".to_string(),
            })),
            event_tx,
        }
    }
    
    pub fn subscribe(&self) -> broadcast::Receiver<Response> {
        self.event_tx.subscribe()
    }
    
    pub async fn is_connected(&self) -> bool {
        self.state.lock().await.writer.is_some()
    }
    
    pub async fn set_user_id(&self, user_id: u64) {
        self.state.lock().await.user_id = Some(user_id);
    }
    
    pub async fn get_token(&self) -> Option<String> {
        self.state.lock().await.token.clone()
    }
    
    pub async fn connect(&self, device_id: String, mt_instance: String, is_mobile: bool) -> ClientResult<Response> {
        let _ = ring::default_provider().install_default();
        let (writer, reader): (Box<dyn TransportWriter>, Box<dyn TransportReader>) = if is_mobile {
            info!("Подключение Mobile TCP/TLS...");
            let transport = MobileTransport::connect_tls(Constants::MOBILE_HOST, 443).await?;
            let (w, r) = transport.split();
            (Box::new(w), Box::new(r))
        } else {
            info!("Подключение к WebSocket...");
            let req_builder = HttpRequestBuilder::new()
                .header("Origin", HeaderValue::from_static(Constants::ORIGIN_HEADER))
                .header("User-Agent", HeaderValue::from_static(Constants::USER_AGENT));

            let ws = WebSocket::connect(Constants::WEBSOCKET_URI.parse().unwrap())
                .with_options(Options::default().with_compression_level(CompressionLevel::fast()))
                .with_request(req_builder)
                .await
                .map_err(|e| Error::ConnectionFailed(e.to_string()))?;

            info!("WebSocket подключен.");
            let transport = WebTransport::new(ws);
            let (w, r) = transport.split();
            (Box::new(w), Box::new(r))
        };

        info!("Разделение потоков и запуск задач...");

        let state_clone = Arc::clone(&self.state);
        let mut state_lock = state_clone.lock().await;
        
        let pending_clone = Arc::clone(&state_lock.pending);
        let (shutdown_tx, shutdown_rx_read) = broadcast::channel(1);
        let shutdown_rx_ping = shutdown_tx.subscribe();
        let event_tx = self.event_tx.clone();
        
        state_lock.device_id = Some(device_id.clone());
        state_lock.mt_instance = Some(mt_instance.clone());
        state_lock.session_id = Utc::now().timestamp_millis();

        tokio::spawn(Self::read_task(reader, pending_clone, event_tx, shutdown_rx_read, Arc::clone(&self.state)));
        debug!("Задача чтения (read_task) запущена.");
        
        let ping_client = self.clone();
        tokio::spawn(Self::ping_task(ping_client, shutdown_rx_ping));
        debug!("Задача пинга (ping_task) запущена.");
        
        state_lock.writer = Some(writer);
        state_lock.shutdown_tx = Some(shutdown_tx);
        
        drop(state_lock);
        
        debug!("Отправка Handshake с deviceId: {}", device_id);
        
        let handshake_payload = if is_mobile {
            let user_agent = json!({
                "deviceType": "ANDROID",
                "appVersion": "25.10.0",
                "osVersion": "Android 13",
                "timezone": "GMT",
                "screen": "130dpi 130dpi 600x874",
                "pushDeviceType": "GCM",
                "locale": "ru",
                "buildNumber": 6401,
                "deviceName": "unknown Generic Android-x86_64",
                "deviceLocale": "ru",
            });
            json!({
                "clientSessionId": 1,
                "mt_instanceid": mt_instance, //Uuid::new_v4().to_string()
                "userAgent": user_agent,
                "deviceId": device_id,
            })
        } else {
            let user_agent = json!({
                "deviceType": "WEB", "locale": "ru", "deviceLocale": "ru", "osVersion": "Linux",
                "deviceName": "Chrome", 
                "headerUserAgent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36",
                "appVersion": "25.10.13",
                "screen": "1080x1920 1.0x", "timezone": "Europe/Moscow",
            });
            json!({
                "deviceId": device_id,
                "userAgent": user_agent,
            })
        };

        self.send_and_wait(6, handshake_payload, 0).await
    }
    
    pub async fn disconnect(&self) {
        let mut state = self.state.lock().await;
        
        if let Some(shutdown_tx) = state.shutdown_tx.take() {
            let _ = shutdown_tx.send(()); 
        }
        
        state.writer = None;
        
        state.pending.lock().unwrap().clear();
        
        state.token = None;
        state.user_id = None;
        state.seq = 0;
        
        state.session_id = Utc::now().timestamp_millis();
        
        info!("Клиент отключен, состояние сброшено.");
    }
    
    fn send_frame(&self, request: Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = ClientResult<()>> + Send>> {
        let this = self.clone();
        
        Box::pin(async move {
            let (should_reconnect, device_id, mt_instance) = {
                let state = this.state.lock().await;
                if state.writer.is_none() {
                    (true, state.device_id.clone(), state.mt_instance.clone())
                } else {
                    (false, None, None)
                }
            };

            if should_reconnect {
                if let (Some(id), Some(mt)) = (device_id, mt_instance) {
                    debug!("Реконнект внутри send_frame...");
                    this.connect(id, mt, true).await?; 
                } else {
                    return Err(Error::ConnectionFailed("Device ID not set".to_string()));
                }
            }

            let mut state = this.state.lock().await;
            if let Some(writer) = &mut state.writer {
                writer.send(request).await
            } else {
                Err(Error::NotConnected)
            }
        })
    }

    pub async fn send_and_wait(
        &self,
        opcode: u16,
        payload: serde_json::Value,
        cmd: u8,
    ) -> ClientResult<Response> {
        let (tx, rx) = oneshot::channel();
        
        let request = {
            let mut state = self.state.lock().await;
            state.seq += 1;
            let current_seq = state.seq;
            
            state.pending.lock().unwrap().insert(current_seq, tx);
            
            Request {
                ver: 11,
                cmd,
                seq: current_seq,
                opcode,
                payload,
            }
        };

        self.send_frame(request.clone()).await?;

        match timeout(Constants::DEFAULT_TIMEOUT, rx).await {
            Ok(Ok(Ok(response))) => {
                trace!("Получен ответ для seq: {}", response.seq);
                if response.payload.get("error").is_some() {
                    Err(Error::ApiResponse(response.payload))
                } else {
                    Ok(response)
                }
            }
            Ok(Ok(Err(e))) => Err(e),
            Ok(Err(e)) => {
                error!("Ошибка получения ответа (канал закрыт) для seq: {}", request.seq);
                Err(e.into())
            }
            Err(_) => {
                warn!("Таймаут запроса для seq: {}", request.seq);
                self.state.lock().await.pending.lock().unwrap().remove(&request.seq);
                Err(Error::RequestTimeout(Constants::DEFAULT_TIMEOUT))
            }
        }
    }
    
    async fn read_task(
        mut reader: Box<dyn TransportReader>,
        pending: Arc<Mutex<HashMap<u64, oneshot::Sender<ClientResult<Response>>>>>,
        event_sender: broadcast::Sender<Response>,
        mut shutdown_rx: broadcast::Receiver<()>,
        state: Arc<TokioMutex<ClientState>>,
    ) {
        loop {
            tokio::select! {
                msg_result = reader.next_message() => {
                    match msg_result {
                        Ok(Some(resp)) => {
                            let seq = resp.seq;
                            let waiting_sender = {
                                let mut guard = pending.lock().unwrap();
                                guard.remove(&seq)
                            };
                            
                            if let Some(sender) = waiting_sender {
                                let _ = sender.send(Ok(resp));
                            } else {
                                let _ = event_sender.send(resp); 
                            }
                        },
                        Ok(None) => {
                            info!("Соединение закрыто (EOF)");
                            let mut s = state.lock().await;
                            s.writer = None;
                            let mut pending_guard = pending.lock().unwrap();
                            for (_, sender) in pending_guard.drain() {
                                let _ = sender.send(Err(Error::ConnectionClosed("Соединение закрыто".to_string())));
                            }
                            break;
                        },
                        Err(e) => {
                            error!("Ошибка чтения транспорта:\n{}", e);
                            let mut s = state.lock().await;
                            s.writer = None;
                            let mut pending_guard = pending.lock().unwrap();
                            for (_, sender) in pending_guard.drain() {
                                let _ = sender.send(Err(Error::ConnectionClosed(e.to_string()))); 
                            }
                            break;
                        }
                    }
                }
                _ = shutdown_rx.recv() => break,
            }
        }
    }
    
    async fn ping_task(
        client: MaxClient,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) {
        info!("Ping task started");
        let mut interval = tokio::time::interval(Constants::PING_INTERVAL);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    debug!("Отправка Ping...");
                    match client.send_and_wait(1, json!({ "interactive": true }), 0).await {
                        Ok(_) => {
                            info!("Pong получен");
                        }
                        Err(e) => {
                            error!("Ошибка Ping: {}. Остановка ping_task", e);
                            break;
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("Получен сигнал завершения для ping_task");
                    break;
                }
            }
        }
        info!("Ping task finished");
    }
    
    pub async fn set_token(&self, token: String) {
        self.state.lock().await.token = Some(token);
    }
    
    pub async fn set_temp_token(&self, token: String) {
        self.state.lock().await.temp_token = Some(token);
    }
}
