use crate::{errors::ClientResult, MaxClient};
use serde_json::{json, Map, Value};
use crate::models::Response;

impl MaxClient {
    pub async fn search_public(
        &self,
        query: String,
        count: i32,
        search_type: String,
    ) -> ClientResult<Response> {
        let payload = json!({
            "query": query,
            "count": count,
            "type": search_type,
        });
        self.send_and_wait(60, payload, 0).await
    }

    pub async fn search_msg(
        &self,
        query: String,
        count: i32,
        marker: Option<String>,
    ) -> ClientResult<Response> {
        let mut payload = Map::new();

        payload.insert("query".into(), json!(query));
        payload.insert("count".into(), json!(count));

        if let Some(m) = marker {
            payload.insert("marker".into(), json!(m));
        }

        self.send_and_wait(68, Value::Object(payload), 0).await
    }

    pub async fn get_chats(
        &self,
        chat_ids: Vec<i64>
    ) -> ClientResult<Response> {
        let payload = json!({
            "chatIds": chat_ids,
        });
        self.send_and_wait(48, payload, 0).await
    }
}
