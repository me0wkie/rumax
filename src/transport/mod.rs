// Объявляем подмодули (файлы web.rs и mobile.rs должны лежать рядом)
pub mod mobile;
pub mod web;

use crate::models::{Request, Response};
use crate::errors::ClientResult;
use async_trait::async_trait;

pub trait TransportFactory: Send {
    type Reader: TransportReader;
    type Writer: TransportWriter;
    fn split(self) -> (Self::Writer, Self::Reader);
}

#[async_trait]
pub trait TransportWriter: Send + Sync {
    async fn send(&mut self, request: Request) -> ClientResult<()>;
}

/// Интерфейс для чтения ответов.
#[async_trait]
pub trait TransportReader: Send + Sync {
    async fn next_message(&mut self) -> ClientResult<Option<Response>>;
}
