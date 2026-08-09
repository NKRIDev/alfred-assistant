use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};
use base64::{engine::general_purpose, Engine};

#[derive(Clone)]
pub struct SpotifyService {
    client_id: String,
    client_secret: String,
    refresh_token: String,
    cached_token: Arc<Mutex<Option<(String, Instant)>>>,
}

impl SpotifyService {
    pub fn new(client_id: String, client_secret: String, refresh_token: String) -> Self {
        Self {
            client_id,
            client_secret,
            refresh_token,
            cached_token: Arc::new(Mutex::new(None)),
        }
    }

    /*
    Reuses the cached token if it is still valid
     */
    async fn get_access_token(&self) -> Result<String, String> {
        let mut cache = self.cached_token.lock().await;

        if let Some((token, expires_at)) = cache.as_ref() {
            if Instant::now() < *expires_at {
                return Ok(token.clone());
            }
        }

        let client = Client::new();
        let auth = general_purpose::STANDARD.encode(format!("{}:{}", self.client_id, self.client_secret));

        let res = client
            .post("https://accounts.spotify.com/api/token")
            .header("Authorization", format!("Basic {}", auth))
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", self.refresh_token.as_str()),
            ])
            .send()
            .await
            .map_err(|e| format!("Spotify token error: {}", e))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_body = res.text().await.unwrap_or_default();
            return Err(format!("Spotify token refresh failed ({}): {}", status, err_body));
        }

        let body: Value = res.json().await
            .map_err(|e| format!("Spotify token parse error: {}", e))?;

        let access_token = body["access_token"].as_str()
            .map(String::from)
            .ok_or_else(|| "Pas d'access_token dans la réponse".to_string())?;

        let expires_in = body["expires_in"].as_u64().unwrap_or(3600);
        let expires_at = Instant::now() + Duration::from_secs(expires_in.saturating_sub(300));

        *cache = Some((access_token.clone(), expires_at));
        Ok(access_token)
    }

    pub async fn play(&self) -> Result<String, String> {
        let device_id = self.ensure_active_device().await?;
        let token = self.get_access_token().await?;
        let client = Client::new();
        let res = client
            .put("https://api.spotify.com/v1/me/player/play")
            .bearer_auth(token)
            .query(&[("device_id", device_id.as_str())])
            .send()
            .await
            .map_err(|e| format!("Spotify play error: {}", e))?;

        self.check_status(res, "Lecture démarrée.").await
    }

    pub async fn pause(&self) -> Result<String, String> {
        let token = self.get_access_token().await?;
        let client = Client::new();
        let res = client
            .put("https://api.spotify.com/v1/me/player/pause")
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| format!("Spotify pause error: {}", e))?;

        self.check_status(res, "Lecture en pause.").await
    }

    pub async fn next_track(&self) -> Result<String, String> {
        let token = self.get_access_token().await?;
        let client = Client::new();
        let res = client
            .post("https://api.spotify.com/v1/me/player/next")
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| format!("Spotify next error: {}", e))?;

        self.check_status(res, "Piste suivante.").await
    }

    pub async fn search_and_play(&self, query: &str) -> Result<String, String> {
        let device_id = self.ensure_active_device().await?;
        let token = self.get_access_token().await?;
        let client = Client::new();

        let search_res = client
            .get("https://api.spotify.com/v1/search")
            .bearer_auth(&token)
            .query(&[("q", query), ("type", "track"), ("limit", "1")])
            .send()
            .await
            .map_err(|e| format!("Spotify search error: {}", e))?;

        let search_body: Value = search_res.json().await
            .map_err(|e| format!("Spotify search parse error: {}", e))?;

        let track_uri = search_body["tracks"]["items"][0]["uri"].as_str()
            .ok_or_else(|| format!("Aucun résultat pour '{}'", query))?;
        let track_name = search_body["tracks"]["items"][0]["name"].as_str().unwrap_or("(inconnu)");

        let play_res = client
            .put("https://api.spotify.com/v1/me/player/play")
            .bearer_auth(&token)
            .query(&[("device_id", device_id.as_str())])
            .json(&serde_json::json!({ "uris": [track_uri] }))
            .send()
            .await
            .map_err(|e| format!("Spotify play error: {}", e))?;

        self.check_status(play_res, &format!("Lecture de '{}' lancée.", track_name)).await
    }

    pub async fn list_devices(&self) -> Result<Vec<Value>, String> {
        let token = self.get_access_token().await?;
        let client = Client::new();

        let res = client
            .get("https://api.spotify.com/v1/me/player/devices")
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| format!("Spotify devices error: {}", e))?;

        let body: Value = res.json().await
            .map_err(|e| format!("Spotify devices parse error: {}", e))?;

        Ok(body["devices"].as_array().cloned().unwrap_or_default())
    }

    /*
    Checks if a Spotify device is available. If none is found, it launches the desktop
    application and waits for it to register with the API (up to ~10s)
    before returning its ID.
     */
    async fn ensure_active_device(&self) -> Result<String, String> {
        let devices = self.list_devices().await?;

        if let Some(device) = devices.first() {
            return device["id"].as_str()
                .map(String::from)
                .ok_or_else(|| "Device sans ID".to_string());
        }

        std::process::Command::new("cmd")
            .args(["/C", "start", "spotify:"])
            .spawn()
            .map_err(|e| format!("Impossible de lancer Spotify: {}", e))?;

        for _ in 0..10 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let devices = self.list_devices().await?;
            if let Some(device) = devices.first() {
                return device["id"].as_str()
                    .map(String::from)
                    .ok_or_else(|| "Device sans ID".to_string());
            }
        }

        Err("Spotify a été lancé mais aucun appareil ne s'est activé à temps.".to_string())
    }

    async fn check_status(&self, res: reqwest::Response, success_msg: &str) -> Result<String, String> {
        if res.status().is_success() || res.status().as_u16() == 204 {
            Ok(success_msg.to_string())
        } else {
            let status = res.status();
            let err_body = res.text().await.unwrap_or_default();
            Err(format!("Échec Spotify ({}): {}", status, err_body))
        }
    }
}