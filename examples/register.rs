use rumax::MaxClient;
use std::io::{self, Write};
use std::fs;
use uuid::Uuid;
use log::{info, error, debug};
use std::sync::Arc;

const DEVICE_ID_FILE: &str = ".device.id";

fn read_line(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn get_device_id() -> String {
    match fs::read_to_string(DEVICE_ID_FILE) {
        Ok(id) if !id.is_empty() => {
            info!("Используем существующий device_id из файла {}", DEVICE_ID_FILE);
            id
        },
        _ => {
            info!("Создаем новый device_id...");
            let new_id = Uuid::new_v4().to_string().replace("-", "");
            fs::write(DEVICE_ID_FILE, &new_id).expect("Не удалось записать .device.id");
            info!("Новый device_id сохранен в {}", DEVICE_ID_FILE);
            new_id
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info,max_client_lib=debug")
    ).init();
    
    let client = Arc::new(MaxClient::new());
    
    let device_id = get_device_id();
    
    info!("Подключение к WebSocket...");
    match client.connect(device_id, true).await {
        Ok(resp) => {
            info!("Handshake успешен!");
            debug!("Ответ Handshake: {:?}", resp.payload);
        }
        Err(e) => {
            error!("Ошибка подключения: {}", e);
            return;
        }
    }
    
    let phone = read_line("Введите номер телефона (+7...): ");
    info!("Отправляем запрос на номер {}", phone);
    
    match client.start_auth(phone).await {
        Ok(resp) => {
            info!("Запрос кода успешен.");
            debug!("Ответ start_auth: {:?}", resp.payload);
        }
        Err(e) => {
            error!("Ошибка запроса кода: {}", e);
            return;
        }
    }
    
    let code = read_line("Введите код из СМС: ");
    info!("Проверяем код...");
    
    let token: String;
    
    match client.check_code(code).await {
        Ok(resp) => {
            info!("Верный код! Регистрируемся...");
            token = resp.payload.get("tokenAttrs")
                .and_then(|t| t.get("REGISTER"))
                .and_then(|l| l.get("token"))
                .and_then(|t| t.as_str())
                .map(|t| t.to_string())
                .unwrap_or_else(|| {
                    log::error!("token отсутствует в ответе сервера!");
                    std::process::exit(1);
                })
        }
        Err(e) => {
            info!("Ошибка проверки кода! {}", e);
            return;
        }
    }
    
    log::info!("token {:?}", token);
    
    let first_name = read_line("Введите имя: ");
    
    let reg_resp = client.submit_register(first_name, Option::None, token).await;
    
    log::info!("reg_resp {:?}", reg_resp);
    
    info!("Выполняем синхронизацию (sync)...");
    match client.sync().await {
        Ok(sync_resp) => {
            log::info!("Синхронизация успешна. {:?}", sync_resp.payload);
            
            let user_id = sync_resp.payload
                .get("profile")
                .and_then(|s| s.get("contact"))
                .and_then(|s| s.get("id"))
                .and_then(|id| id.as_u64());
            
            log::info!("test {:?}", user_id);
            
            if let Some(id) = user_id {
                log::info!("Установка user_id: {}", id);
                client.set_user_id(id).await;
                
                log::info!("Запуск фоновой задачи телеметрии...");
                client.spawn_telemetry_task().await;
            } else {
                log::warn!("Не удалось найти user_id в ответе sync. Телеметрия не запущена.");
            }
        }
        Err(e) => {
            log::error!("Ошибка sync: {}", e);
            return;
        }
    }
    
    info!("\nУспешная регистрация!");
    
    read_line("");
    info!("Завершение работы...");
}

