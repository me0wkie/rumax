use rumax::{MaxClient, models::Response};
use std::io::{self, Write};
use std::fs;
use std::sync::Arc;
use uuid::Uuid;
use log::{info, error, debug, warn};

const DEVICE_ID_FILE: &str = ".device.id";
const TOKEN_FILE: &str = ".session.token"; // <-- Файл для токена

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
            info!("Используем существующий device_id из {}", DEVICE_ID_FILE);
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

fn load_token() -> Option<String> {
    match fs::read_to_string(TOKEN_FILE) {
        Ok(token) if !token.is_empty() => {
            info!("Токен сессии загружен из {}", TOKEN_FILE);
            Some(token)
        }
        _ => {
            info!("Файл токена {} не найден", TOKEN_FILE);
            None
        }
    }
}

fn save_token(token: &str) {
    if let Err(e) = fs::write(TOKEN_FILE, token) {
        error!("Не удалось сохранить токен в {}: {}", TOKEN_FILE, e);
    } else {
        info!("Токен сессии сохранен в {}", TOKEN_FILE);
    }
}

fn delete_token() {
    if fs::remove_file(TOKEN_FILE).is_ok() {
        info!("Файл токена {} удален", TOKEN_FILE);
    }
}

async fn set_user_id_and_spawn_telemetry(client: &MaxClient, sync_resp: &Response) {
    let user_id = sync_resp.payload
        .get("profile")
        .and_then(|s| s.get("contact"))
        .and_then(|s| s.get("id"))
        .and_then(|id| id.as_u64());
    
    if let Some(id) = user_id {
        info!("Установка user_id: {}", id);
        client.set_user_id(id).await;
        
        info!("Запуск фоновой задачи телеметрии...");
        client.spawn_telemetry_task().await;
    } else {
        warn!("Не удалось найти user_id в ответе sync. Телеметрия не запущена!");
    }
}


#[tokio::main]
async fn main() -> Result<(), rumax::errors::Error> {
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
            return Err(e.into());
        }
    }

    if let Some(token) = load_token() {
        info!("Попытка входа по сохраненному токену...");
        
        client.set_token(token).await;

        match client.sync().await {
            Ok(sync_resp) => {
                info!("Вход по токену успешен!");
                set_user_id_and_spawn_telemetry(&client, &sync_resp).await;
            }
            Err(e) => {
                warn!("Ошибка входа по токену (возможно, истек): {}. Удаляем токен", e);
                delete_token();
                info!("Перезапустите скрипт для входа по номеру телефона");
                return Ok(());
            }
        }
    } else {
        info!("Токен не найден, запуск входа по номеру телефона...");
        
        let phone = read_line("Введите номер телефона (+7...): ");
        if let Err(e) = client.start_auth(phone).await {
            error!("Ошибка запроса кода: {}", e);
            return Err(e.into());
        }
        
        let code = read_line("Введите код из СМС/звонка: ");
        if let Err(e) = client.check_code(code).await {
            error!("Ошибка проверки кода: {}", e);
            return Err(e.into());
        }
        
        match client.sync().await {
            Ok(sync_resp) => {
                info!("Вход по коду и телефону успешен");
                
                if let Some(new_token) = client.get_token().await {
                    save_token(&new_token);
                } else {
                    warn!("Не удалось получить токен из клиента для сохранения");
                }

                set_user_id_and_spawn_telemetry(&client, &sync_resp).await;
            }
            Err(e) => {
                error!("Ошибка синхронизации: {}", e);
                return Err(e.into());
            }
        }
    }
    
    info!("Успешный вход!");
    
    let chat_id_str = read_line("Введите Chat ID для тестового сообщения: ");
    
    let chat_id: u64 = match chat_id_str.parse() {
        Ok(num) => num,
        Err(_) => {
            error!("Это не похоже на число (u64). Выходим");
            return Ok(());
        }
    };
    
    let message = read_line("Введите текст сообщения: ");
    
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

    info!("\nКлиент остается подключенным. Нажмите Enter для выхода");
    read_line("");
    info!("Завершение работы...");

    Ok(())
}
