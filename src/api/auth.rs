use crate::{errors::ClientResult, MaxClient};
use crate::models::Response;
use serde_json::json;

impl MaxClient {
    /**
     * Начало логина
     */
    pub async fn start_auth(&self, phone: String) -> ClientResult<Response> {
        let payload = json!({ "phone": phone, "type": "START_AUTH", "language": "ru" });
        let resp = self.send_and_wait(17, payload, 0).await?;
        
        log::info!("start_auth response {:?}", resp);
        
        if let Some(token) = resp.payload.get("token").and_then(|t| t.as_str()) {
            log::info!("temp token from response {:?}", token);
            self.set_temp_token(token.to_string()).await;
        }

        Ok(resp)
    }
    
    /**
     * Завершение логина
     */
    pub async fn check_code(&self, code: String) -> ClientResult<Response> {
        let state = self.state.lock().await;
        let token = state.temp_token.as_ref().ok_or("No temporary token found".to_string())?;
        
        let payload = json!({ "token": token, "verifyCode": code, "authTokenType": "CHECK_CODE" });
        
        drop(state);
        
        let resp = self.send_and_wait(18, payload, 0).await?;
        
        log::info!("check_code response {:?}", resp);
        
        if let Some(token) = resp.payload.get("token").and_then(|t| t.as_str()) {
            if resp.payload.get("tokenType").and_then(|t| t.as_str()).unwrap() == "REGISTER" {
                self.set_temp_token(token.to_string()).await;
            }
            else {
                self.set_token(token.to_string()).await;
            }
        }
        
        Ok(resp)
    }
    
    /**
     * Регистрация
     */
    pub async fn submit_register(
        &self,
        first_name: String,
        last_name: Option<String>,
    ) -> ClientResult<Response> {
        let payload = json!({
            "firstName": first_name,
            "lastName": last_name,
            "photoId": 2981369,
            "avatarType": "PRESET_AVATAR",
            "tokenType": "REGISTER",
        });
        
        let resp = self.send_and_wait(23, payload, 0).await?;
        
        if let Some(token) = resp
            .payload
            .get("token")
            .and_then(|t| t.as_str())
        {
            log::info!("Token received! {:?}", token.to_string());
            self.set_token(token.to_string()).await;
        }
        
        Ok(resp)
    }
    
    /**
     * Перезаход в мессенджер
     */
    pub async fn sync(&self) -> ClientResult<Response> {
        let state = self.state.lock().await;
        let token = state.token.as_ref().ok_or("No token set".to_string())?;
        
        let payload = json!({
            "interactive": true, "token": token,
            "chatsSync": 0, "contactsSync": 0, "presenceSync": 0, "draftsSync": 0, "chatsCount": 40,
        });
        
        // Отпускаем лок перед .await
        drop(state);
        
        self.send_and_wait(19, payload, 0).await
    }
}
