use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};

/*
Manages the request for and receipt of
access tokens from Google.
 */
#[derive(Clone)]
pub struct GoogleAuthService {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: String,
    pub cached_token: Arc<Mutex<Option<(String, Instant)>>>,
}

impl GoogleAuthService {
    pub fn new(client_id: String, client_secret: String, refresh_token: String) -> Self {
        Self {
            client_id,
            client_secret,
            refresh_token,
            cached_token: Arc::new(Mutex::new(None)),
        }
    }

    /*
    Recover a custom access token
     */
    pub async fn get_access_token(&self) -> Result<String, String> {
        let mut cache = self.cached_token.lock().await;

        if let Some((token, expires_at)) = cache.as_ref() {
            if Instant::now() < *expires_at {
                return Ok(token.clone());
            }
        }

        let client = Client::new();
        let res = client
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("refresh_token", self.refresh_token.as_str()),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await
            .map_err(|e| format!("Google token error: {}", e))?;

        let body: Value = res.json().await
            .map_err(|e| format!("Google token parse error: {}", e))?;

        let access_token = body["access_token"].as_str()
            .map(String::from)
            .ok_or_else(|| "Pas d'access_token dans la réponse".to_string())?;

        let expires_in = body["expires_in"].as_u64().unwrap_or(3600);
        let expires_at = Instant::now() + Duration::from_secs(expires_in.saturating_sub(300));

        *cache = Some((access_token.clone(), expires_at));
        Ok(access_token)
    }
}