use crate::{errors::ClientResult, MaxClient};
use serde_json::{json, Map, Value};
use crate::models::Response;

impl MaxClient {
    pub async fn join_channel(
        &self,
        link: String
    ) -> ClientResult<Response> {
        let payload = json!({
            "link": link,
        });
        self.send_and_wait(57, payload, 0).await
    }

    pub async fn quit_channel(
        &self,
        chat_id: i64
    ) -> ClientResult<Response> {
        let payload = json!({
            "chatId": chat_id
        });

        self.send_and_wait(58, payload, 0).await
    }

    pub async fn resolve_channel_by_name(
        &self,
        link: String,
    ) -> ClientResult<Response> {
        let payload = json!({
            "link": format!("https://max.ru/{link}"),
        });
        self.send_and_wait(89, payload, 0).await
    }

    pub async fn get_members(
        &self,
        chat_id: i64,
        count: i64,
        marker: Option<i64>,
    ) -> ClientResult<Response> {
        let mut payload = Map::new();

        payload.insert("type".into(), json!("MEMBER"));

        if let Some(m) = marker {
            payload.insert("marker".into(), json!(m));
        }

        payload.insert("chatId".into(), json!(chat_id));
        payload.insert("count".into(), json!(count));

        self.send_and_wait(59, Value::Object(payload), 0).await
    }

    pub async fn find_members(
        &self,
        chat_id: i64,
        query: String
    ) -> ClientResult<Response> {
        let payload = json!({
            "type": "MEMBER",
            "query": query,
            "chatId": chat_id,
        });
        self.send_and_wait(59, payload, 0).await
    }
}
