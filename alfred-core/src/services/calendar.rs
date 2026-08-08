use reqwest::Client;
use serde_json::{json, Value};
use crate::services::google_auth::GoogleAuthService;

#[derive(Clone)]
pub struct CalendarService {
    auth: GoogleAuthService,
}

pub struct CalendarEvent {
    pub id: String,
    pub summary: String,
    pub start: String,
    pub end: String,
}

impl CalendarService {
    pub fn new(auth: GoogleAuthService) -> Self {
        Self { auth }
    }
    
    pub async fn list_upcoming(&self, max_results: u32) -> Result<Vec<CalendarEvent>, String> {
        let access_token = self.auth.get_access_token().await?;
        let client = Client::new();
        let now = chrono::Utc::now().to_rfc3339();

        let res = client
            .get("https://www.googleapis.com/calendar/v3/calendars/primary/events")
            .bearer_auth(&access_token)
            .query(&[
                ("timeMin", now.as_str()),
                ("maxResults", &max_results.to_string()),
                ("singleEvents", "true"),
                ("orderBy", "startTime"),
            ])
            .send()
            .await
            .map_err(|e| format!("Calendar list error: {}", e))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_body = res.text().await.unwrap_or_default();
            eprintln!("Calendar list error: {}", err_body);
            return Err(format!("Calendar list failed ({}): {}", status, err_body));
        }

        let body: Value = res.json().await
            .map_err(|e| format!("Calendar list parse error: {}", e))?;

        let events = body["items"].as_array().unwrap_or(&vec![]).iter().map(|e| {
            CalendarEvent {
                id: e["id"].as_str().unwrap_or_default().to_string(),
                summary: e["summary"].as_str().unwrap_or("(sans titre)").to_string(),
                start: e["start"]["dateTime"].as_str()
                    .or(e["start"]["date"].as_str())
                    .unwrap_or_default().to_string(),
                end: e["end"]["dateTime"].as_str()
                    .or(e["end"]["date"].as_str())
                    .unwrap_or_default().to_string(),
            }
        }).collect();

        Ok(events)
    }

    pub async fn create_event(&self, summary: &str, start: &str, end: &str, description: &str) -> Result<String, String> {
        let access_token = self.auth.get_access_token().await?;
        let client = Client::new();

        let res = client
            .post("https://www.googleapis.com/calendar/v3/calendars/primary/events")
            .bearer_auth(&access_token)
            .json(&json!({
                "summary": summary,
                "description": description,
                "start": { "dateTime": start },
                "end": { "dateTime": end }
            }))
            .send()
            .await
            .map_err(|e| format!("Calendar create error: {}", e))?;

        if res.status().is_success() {
            Ok(format!("Événement '{}' créé.", summary))
        } else {
            let status = res.status();
            let err_body = res.text().await.unwrap_or_default();
            eprint!("{}", err_body);
            Err(format!("Échec création ({}): {}", status, err_body))
        }
    }

    pub async fn update_event(&self, event_id: &str, summary: Option<&str>, start: Option<&str>, end: Option<&str>) -> Result<String, String> {
        let access_token = self.auth.get_access_token().await?;
        let client = Client::new();

        let mut body = json!({});
        if let Some(s) = summary { body["summary"] = json!(s); }
        if let Some(s) = start { body["start"] = json!({ "dateTime": s }); }
        if let Some(e) = end { body["end"] = json!({ "dateTime": e }); }

        let res = client
            .patch(format!("https://www.googleapis.com/calendar/v3/calendars/primary/events/{}", event_id))
            .bearer_auth(&access_token)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Calendar update error: {}", e))?;

        if res.status().is_success() {
            Ok("Événement mis à jour.".to_string())
        } else {
            let status = res.status();
            let err_body = res.text().await.unwrap_or_default();
            eprint!("{}", err_body);
            Err(format!("Échec mise à jour ({}): {}", status, err_body))
        }
    }

    pub async fn delete_event(&self, event_id: &str) -> Result<String, String> {
        let access_token = self.auth.get_access_token().await?;
        let client = Client::new();

        let res = client
            .delete(format!("https://www.googleapis.com/calendar/v3/calendars/primary/events/{}", event_id))
            .bearer_auth(&access_token)
            .send()
            .await
            .map_err(|e| format!("Calendar delete error: {}", e))?;

        if res.status().is_success() {
            Ok("Événement supprimé.".to_string())
        } else {
            let status = res.status();
            let err_body = res.text().await.unwrap_or_default();
            eprint!("{}", err_body);
            Err(format!("Échec suppression ({}): {}", status, err_body))
        }
    }
}