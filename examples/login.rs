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

fn read_tokens() -> Option<(String, String)> {
    if let Ok(content) = fs::read_to_string(DEVICE_ID_FILE) {
        let mut lines = content.lines().map(str::trim).filter(|l| !l.is_empty());
        if let (Some(first), Some(second)) = (lines.next(), lines.next()) {
            return Some((first.to_string(), second.to_string()));
        }
    }
    None
}

fn write_tokens(id1: &str, id2: &str) {
    let content = format!("{}\n{}", id1, id2);
    fs::write(DEVICE_ID_FILE, content)
        .expect("Не удалось записать .device.id");
}

fn get_device() -> (String, String) {
    match read_tokens() {
        Some((id1, id2)) => {
            info!("Используем существующие device_id из файла {}", DEVICE_ID_FILE);
            (id1, id2)
        }
        None => {
            info!("Создаем новые device_id...");

            let id1 = Uuid::new_v4().to_string().replace("-", "");
            let id2 = Uuid::new_v4().to_string();

            write_tokens(&id1, &id2);

            info!("Новые device_id сохранены в {}", DEVICE_ID_FILE);
            (id1, id2)
        }
    }
}


#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info,max_client_lib=debug")
    ).init();

    let client = Arc::new(MaxClient::new());

    let (device_id, mt) = get_device_id();

    info!("Подключение к WebSocket...");
    match client.connect(device_id, mt, true).await {
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
            info!("Код принят, логин успешен!");
            debug!("Ответ check_code: {:?}", resp.payload);
            
            token = resp.payload.get("tokenAttrs")
                .and_then(|t| t.get("LOGIN"))
                .and_then(|l| l.get("token"))
                .and_then(|t| t.as_str())
                .map(|t| t.to_string())
                .unwrap_or_else(|| {
                    log::error!("token отсутствует в ответе сервера!");
                    std::process::exit(1);
                });
        }
        Err(e) => {
            error!("Ошибка проверки кода: {}", e);
            return;
        }
    }
    
    info!("Выполняем синхронизацию...");
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
        }
    }
    
    info!("\nУспешный вход!");
    
    let chat_id_str = read_line("Введите Chat ID для тестового сообщения: ");
    
    let chat_id: u64 = match chat_id_str.parse() {
        Ok(num) => num,
        Err(_) => {
            error!("Это не похоже на число (u64). Выход.");
            return;
        }
    };
    
    let message = read_line("Введите текст сообщения: ");
    
    info!("Отправляем сообщение в чат {}...", chat_id);
    match client.send_message(chat_id, message, None).await {
        Ok(resp) => {
            info!("Сообщение успешно отправлено!");
            info!("Ответ send_message: {:?}", resp.payload);
        }
        Err(e) => {
            error!("Ошибка отправки сообщения: {}", e);
        }
    }
    
    match client.fetch_history(chat_id, Option::None, 0, 200).await {
        Ok(resp) => {
            info!("Ответ fetch_history: {:?}", resp.payload);
        }
        Err(e) => {
            error!("Ошибка отправки сообщения: {}", e);
        }
    }
    
    info!("\nКлиент остается подключенным. Нажмите Enter для выхода.");
    read_line("");
    info!("Завершение работы...");
}
