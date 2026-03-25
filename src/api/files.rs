use crate::{errors::ClientResult, MaxClient};
use crate::models::Response;
use serde_json::{json, Value};
use tokio::{fs::File};
use tokio_util::io::ReaderStream;
use reqwest::{Client, Body, multipart};
use futures_util::StreamExt;
use std::time::Duration;
use std::fs;

const CHUNK_SIZE: usize = 6 * 1024 * 1024;

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
    ) -> Value {
        let ext = match path.split('.').last() {
            Some(e) => e.to_lowercase(),
            None => return json!({ "error": "Failed to get file extension" }),
        };

        let mime = match ext.as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            _ => return json!({ "error": "Photo validation failed" }),
        };

        let file_bytes = match fs::read(&path) {
            Ok(b) => b,
            Err(_) => return json!({ "error": "Failed to read photo" }),
        };

        let form = multipart::Form::new()
        .part("file", multipart::Part::bytes(file_bytes)
        .file_name(format!("image.{}", ext))
        .mime_str(mime).unwrap());

        let client = reqwest::Client::new();
        let response = match client.post(upload_url).multipart(form).send().await {
            Ok(r) => r,
            Err(e) => return json!({ "error": format!("Upload request failed: {}", e) }),
        };

        if !response.status().is_success() {
            return json!({ "error": format!("Upload failed with status {}", response.status()) });
        }

        let json_resp: serde_json::Value = match response.json().await {
            Ok(j) => j,
            Err(e) => return json!({ "error": format!("Failed to parse JSON: {}", e) }),
        };

        let token = match json_resp.get("photos")
        .and_then(|v| v.as_object())
        .and_then(|obj| obj.values().next())
        .and_then(|photo| photo.get("token"))
        .and_then(|t| t.as_str()) {
            Some(t) => t,
            None => return json!({ "error": "Token not found in response" }),
        };

        json!({ "photo_token": token.to_string() })
    }

    pub async fn upload_video(
        &self,
        upload_url: String,
        video_id: u64,
        token: String,
        path: String,
    ) -> Value {
        // читаем весь файл в память
        let file_bytes = match fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                return json!({ "error": format!("Failed to read video: {}", e) });
            }
        };

        let file_size = file_bytes.len();
        if file_size == 0 {
            return json!({ "error": "Empty file" });
        }

        let file_name = path.split('/').last().unwrap_or("video.mp4");

        let client = match Client::builder()
        .timeout(Duration::from_secs(600))
        .build()
        {
            Ok(c) => c,
            Err(e) => return json!({ "error": format!("Failed to build client: {}", e) }),
        };

        let response = match client.post(upload_url)
        .header("Content-Disposition", format!("attachment; filename={}", file_name))
        .header("Content-Range", format!("0-{}/{}", file_size - 1, file_size))
        .header("Content-Length", file_size)
        .header("Connection", "keep-alive")
        .body(file_bytes)
        .send()
        .await
        {
            Ok(r) => r,
            Err(e) => return json!({ "error": format!("Upload failed: {}", e) }),
        };

        if !response.status().is_success() {
            return json!({ "error": format!("Upload failed with status {}", response.status()) });
        }

        json!({
            "videoId": video_id,
            "token": token
        })
    }

    pub async fn upload_file(
        &self,
        upload_url: String,
        file_id: u64,
        path: String,
    ) -> Value {
        let file = match File::open(&path).await {
            Ok(f) => f,
            Err(e) => return json!({ "error": format!("Failed to open file: {}", e) }),
        };

        let metadata = match file.metadata().await {
            Ok(m) => m,
            Err(e) => return json!({ "error": format!("Failed to get metadata: {}", e) }),
        };

        let file_size = metadata.len();
        if file_size == 0 {
            return json!({ "error": "Empty file" });
        }

        let file_name = path.split('/').last().unwrap_or("file");

        let stream = ReaderStream::new(file)
        .map(|chunk_result| {
            chunk_result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        });

        let body = Body::wrap_stream(stream);

        let client = match Client::builder().build() {
            Ok(c) => c,
            Err(e) => return json!({ "error": format!("Client build failed: {}", e) }),
        };

        let response = match client
        .post(upload_url)
        .header("Content-Disposition", format!("attachment; filename={}", file_name))
        .header("Content-Length", file_size)
        .header("Content-Range", format!("0-{}/{}", file_size - 1, file_size))
        .body(body)
        .send()
        .await
        {
            Ok(r) => r,
            Err(e) => return json!({ "error": format!("Upload failed: {}", e) }),
        };

        if !response.status().is_success() {
            return json!({
                "error": format!("Upload failed with status {}", response.status())
            });
        }

        json!({
            "fileId": file_id
        })
    }
}
