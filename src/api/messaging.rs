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
                "elements": args_map.get("elements").cloned().unwrap_or(json!([])),
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
        message_id: u64,
        reaction: String
    ) -> ClientResult<Response> {
        let payload = json!({
            "chatId": chat_id,
            "messageId": message_id,
            "reaction": {
                "reactionType": "EMOJI",
                "id": reaction,
            }
        });
        self.send_and_wait(178, payload, 0).await
    }
    
    pub async fn remove_reaction(
        &self,
        chat_id: i64,
        message_id: u64,
    ) -> ClientResult<Response> {
        let payload = json!({
            "chatId": chat_id,
            "messageId": message_id,
        });
        self.send_and_wait(179, payload, 0).await
    }

    pub async fn get_video_by_id(
        &self,
        chat_id: i64,
        message_id: u64,
        video_id: i64,
    ) -> ClientResult<Response> {
        let payload = json!({
            "chatId": chat_id,
            "messageId": message_id,
            "videoId": video_id
        });
        self.send_and_wait(83, payload, 0).await
    }

    pub async fn get_file_by_id(
        &self,
        chat_id: i64,
        message_id: u64,
        file_id: i64,
    ) -> ClientResult<Response> {
        let payload = json!({
            "chatId": chat_id,
            "messageId": message_id,
            "fileId": file_id
        });
        self.send_and_wait(88, payload, 0).await
    }

    pub async fn read_message(
        &self,
        chat_id: i64,
        message_id: u64,
    ) -> ClientResult<Response> {
        let payload = json!({
            "type": "READ_MESSAGE",
            "chatId": chat_id,
            "messageId": message_id,
            "mark": Utc::now().timestamp_millis() as u64,
        });
        self.send_and_wait(50, payload, 0).await
    }

    pub async fn pin_message(
        &self,
        chat_id: i64,
        message_id: u64,
        notify_pin: bool,
    ) -> ClientResult<Response> {
        let payload = json!({
            "chatId": chat_id,
            "notifyPin": notify_pin,
            "pinMessageId": message_id
        });
        self.send_and_wait(55, payload, 0).await
    }

    pub async fn delete_message(
        &self,
        chat_id: i64,
        message_id: u64,
        for_me: bool,
    ) -> ClientResult<Response> {
        let payload = json!({
            "chatId": chat_id,
            "messageId": message_id,
            "forMe": for_me,
        });
        self.send_and_wait(66, payload, 0).await
    }

    pub async fn edit_message(
        &self,
        chat_id: i64,
        message_id: u64,
        text: String
    ) -> ClientResult<Response> {
        let payload = json!({
            "chatId": chat_id,
            "messageId": message_id,
            "text": text,
            "elements": [],
            "attaches": [],
        });
        self.send_and_wait(67, payload, 0).await
    }
    
    /* TODO Upload image, video, file */
    
    
}
