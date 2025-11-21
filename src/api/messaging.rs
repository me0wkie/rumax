use crate::{errors::ClientResult, MaxClient};
use crate::models::Response;
use serde_json::json;
use std::collections::HashMap;
use chrono::Utc;

impl MaxClient {
    pub async fn send_message(
        &self,
        chat_id: i64,
        text: String,
        args: Option<HashMap<String, serde_json::Value>>,
    ) -> ClientResult<Response> {
        let args_map = args.unwrap_or_default();
        let payload = json!({
            "chatId": chat_id,
            "message": {
                "text": text,
                "cid": Utc::now().timestamp_millis(),
                "elements": json!([]),
                "attaches": args_map.get("attaches").cloned().unwrap_or(json!([])),
                "link": args_map.get("replyTo").cloned().map(|id| json!({"type": "REPLY", "messageId": id.to_string()})),
            },
            "notify": args_map.get("notify").cloned().unwrap_or(json!(true)),
        });
        self.send_and_wait(64, payload, 0).await
    }
    
    pub async fn fetch_history(
        &self,
        chat_id: i64,
        from_time: Option<u64>,
        forward: u32,
        backward: u32,
    ) -> ClientResult<Response> {
        let payload = json!({
            "chatId": chat_id,
            "from": from_time.unwrap_or(Utc::now().timestamp_millis() as u64),
            "forward": forward,
            "backward": backward,
            "getMessages": true
        });
        self.send_and_wait(49, payload, 0).await
    }
    
    
    pub async fn add_reaction(
        &self,
        chat_id: i64,
        message_id: String,
        reaction: String
    ) -> ClientResult<Response> {
        let payload = json!({
            "chatId": chat_id,
            "messageId": message_id,
            "reaction": {
                "reaction_type": "EMOJI",
                "id": reaction,
            }
        });
        self.send_and_wait(178, payload, 0).await
    }
    
    pub async fn remove_reaction(
        &self,
        chat_id: i64,
        message_id: String,
    ) -> ClientResult<Response> {
        let payload = json!({
            "chatId": chat_id,
            "messageId": message_id,
        });
        self.send_and_wait(179, payload, 0).await
    }
    
    /* TODO Pin, edit, delete */
    
    
}
