use crate::{errors::ClientResult, MaxClient};
use serde_json::{json, Map, Value};
use crate::models::Response;
use chrono::Utc;

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

    pub async fn create_group(
        &self,
        title: String,
        participant_ids: Option<Vec<i64>>,
        notify: Option<bool>
    ) -> ClientResult<Response> {
        let payload = json!({
            "message": {
                "cid": Utc::now().timestamp_millis(),
                "attaches": [{
                    "_type": "CONTROL",
                    "event": "new",
                    "chatType": "CHAT",
                    "title": title,
                    "userIds": participant_ids.unwrap_or_default()
                }]
            },
            "notify": notify.unwrap_or(true)
        });

        self.send_and_wait(64, payload, 0).await
    }

    pub async fn delete_chat(
        &self,
        chat_id: i64,
        last_event_time: Option<i64>,
        for_all: Option<bool>,
    ) -> ClientResult<Response> {
        let payload = json!({
            "chatId": chat_id,
            "lastEventTime": last_event_time.unwrap_or(Utc::now().timestamp_millis()),
            "forAll": for_all.unwrap_or(false)
        });

        self.send_and_wait(52, payload, 0).await
    }

    pub async fn leave_group(
        &self,
        chat_id: i64,
    ) -> ClientResult<Response> {
        let payload = json!({
            "chatId": chat_id
        });

        self.send_and_wait(58, payload, 0).await
    }
}
