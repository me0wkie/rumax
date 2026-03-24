use log::{error};
use crate::{errors::ClientResult, MaxClient};
use crate::models::Response;
use serde_json::{json, Value};
use reqwest::multipart;
use std::fs;

impl MaxClient {
    /*
     * Returns url, file_id
     */
    pub async fn get_photo_upload(
        &self,
        count: i64,
        profile: bool,
    ) -> ClientResult<Response> {
        let payload = json!({
            "count": count,
            "profile": profile,
        });
        self.send_and_wait(80, payload, 0).await
    }

    pub async fn get_video_upload(
        &self,
        count: i64,
        profile: bool,
    ) -> ClientResult<Response> {
        let payload = json!({
            "count": count,
            "profile": profile,
        });
        self.send_and_wait(82, payload, 0).await
    }

    pub async fn get_file_upload(
        &self,
        count: i64,
        profile: bool,
    ) -> ClientResult<Response> {
        let payload = json!({
            "count": count,
            "profile": profile,
        });
        self.send_and_wait(87, payload, 0).await
    }

    pub async fn upload_photo(
        &self,
        upload_url: String,
        path: String
    ) -> Option<String> {
        let ext = match path.split('.').last() {
            Some(e) => e.to_string(),
            None => {
                error!("Failed to get file extension");
                return None;
            }
        };

        let mime = match ext.as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            _ => {
                error!("Photo validation failed");
                return None;
            }
        };

        let file_bytes = match fs::read(path).ok() {
            Some(b) => b,
            None => {
                error!("Failed to read photo");
                return None;
            }
        };

        let form = multipart::Form::new()
            .part(
                "file",
                multipart::Part::bytes(file_bytes)
                    .file_name(format!("image.{}", ext))
                    .mime_str(mime)
                    .unwrap(),
            );

        let client = reqwest::Client::new();

        let response = match client.post(upload_url).multipart(form).send().await {
            Ok(r) => r,
            Err(e) => {
                error!("Upload request failed: {}", e);
                return None;
            }
        };

        if !response.status().is_success() {
            error!("Upload failed with status {}", response.status());
            return None;
        };

        let json: Value = match response.json().await {
            Ok(j) => j,
            Err(e) => {
                error!("Failed to parse JSON response: {}", e);
                return None;
            }
        };

        let photo_data = json.get("photos")?.as_object()?.values().next()?;
        let token = photo_data.get("token")?.as_str()?;

        Some(token.to_string())
    }
}
