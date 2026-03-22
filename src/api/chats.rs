use crate::{errors::ClientResult, MaxClient};
use crate::models::Response;
use serde_json::json;

impl MaxClient {
    pub async fn public_search(
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
