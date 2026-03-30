use crate::{errors::ClientResult, MaxClient};
use crate::models::Response;
use serde_json::json;

impl MaxClient {
    pub async fn get_calls(
        &self,
        forward: bool,
        count: i64
    ) -> ClientResult<Response> {
        let payload = json!({
            "forward": forward,
            "count": count,
        });
        self.send_and_wait(79, payload, 0).await
    }
}
