use std::time::Duration;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum Error {
    NotConnected,
    ConnectionFailed(String),
    ConnectionClosed(String),
    SendFailed(String),
    ParseError(serde_json::Error),
    RequestTimeout(Duration),
    ApiResponse(serde_json::Value),
    OneshotRecvError(oneshot::error::RecvError),
    IoError(std::io::Error),
    TauriError(String),
    Other(String),
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Other(s)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::ParseError(e)
    }
}

impl From<oneshot::error::RecvError> for Error {
    fn from(e: oneshot::error::RecvError) -> Self {
        Error::OneshotRecvError(e)
    }
}

use std::fmt;
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotConnected => write!(f, "Клиент не подключен"),
            Error::ConnectionFailed(e) => write!(f, "Ошибка подключения: {}", e),
            Error::ConnectionClosed(e) => write!(f, "Соединение оборвано: {}", e),
            Error::SendFailed(e) => write!(f, "Ошибка отправки: {}", e),
            Error::ParseError(e) => write!(f, "Ошибка парсинга JSON: {}", e),
            Error::RequestTimeout(d) => write!(f, "Таймаут запроса: {:?}", d),
            Error::ApiResponse(json) => write!(f, "Ошибка API: {}", json),
            Error::OneshotRecvError(e) => write!(f, "Ошибка получения ответа: {}", e),
            Error::IoError(e) => write!(f, "Ошибка I/O: {}", e),
            Error::TauriError(e) => write!(f, "Ошибка Tauri: {}", e),
            Error::Other(s) => write!(f, "Неизвестная ошибка: {}", s),
        }
    }
}

pub type ClientResult<T> = Result<T, Error>;
