use reqwest::Client;
use serde_json::{json, Value};
use base64::{engine::general_purpose, Engine};

/*
Service to communicate with the Google gmail service
 */
#[derive(Clone)]
pub struct GmailService {
    client_id: String,
    client_secret: String,
    refresh_token: String,
}

#[derive(serde::Serialize)]
pub struct EmailSummary {
    pub id: String,
    pub thread_id: String,
    pub from: String,
    pub subject: String,
    pub snippet: String,
}

impl GmailService {
    pub fn new(client_id: String, client_secret: String, refresh_token: String) -> Self {
        Self { client_id, client_secret, refresh_token }
    }

    /*
    Recover access token
     */
    async fn get_access_token(&self) -> Result<String, String> {
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
            .map_err(|e| format!("Gmail token error: {}", e))?;

        let body: Value = res.json().await
            .map_err(|e| format!("Gmail token parse error: {}", e))?;

        body["access_token"].as_str()
            .map(String::from)
            .ok_or_else(|| "Pas d'access_token dans la réponse".to_string())
    }

    /*
    Searches for messages based on a Gmail query (search syntax
    Native Gmail: "is:unread", "from:x@y.com", "subject:invoice", etc.)
    then get the metadata (From, Subject) + snippet for each.
     */
    async fn search_messages(&self, query: &str, max_results: u32) -> Result<Vec<EmailSummary>, String> {
        let access_token = self.get_access_token().await?;
        let client = Client::new();

        let list_res = client
            .get("https://gmail.googleapis.com/gmail/v1/users/me/messages")
            .bearer_auth(&access_token)
            .query(&[("q", query), ("maxResults", &max_results.to_string())])
            .send()
            .await
            .map_err(|e| format!("Gmail list error: {}", e))?;

        if !list_res.status().is_success() {
            let status = list_res.status();
            let err_body = list_res.text().await.unwrap_or_default();
            return Err(format!("Gmail list failed ({}): {}", status, err_body));
        }

        let list_body: Value = list_res.json().await
            .map_err(|e| format!("Gmail list parse error: {}", e))?;

        let message_ids: Vec<String> = list_body["messages"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|m| m["id"].as_str().map(String::from))
            .collect();

        let mut summaries = Vec::new();
        for id in message_ids {
            let detail_res = client
                .get(format!("https://gmail.googleapis.com/gmail/v1/users/me/messages/{}", id))
                .bearer_auth(&access_token)
                .query(&[
                    ("format", "metadata"),
                    ("metadataHeaders", "From"),
                    ("metadataHeaders", "Subject"),
                ])
                .send()
                .await
                .map_err(|e| format!("Gmail get error: {}", e))?;

            let detail: Value = detail_res.json().await
                .map_err(|e| format!("Gmail get parse error: {}", e))?;

            let headers = detail["payload"]["headers"].as_array().cloned().unwrap_or_default();
            let get_header = |name: &str| -> String {
                headers.iter()
                    .find(|h| h["name"].as_str() == Some(name))
                    .and_then(|h| h["value"].as_str())
                    .unwrap_or("(inconnu)")
                    .to_string()
            };

            summaries.push(EmailSummary {
                id: detail["id"].as_str().unwrap_or_default().to_string(),
                thread_id: detail["threadId"].as_str().unwrap_or_default().to_string(),
                from: get_header("From"),
                subject: get_header("Subject"),
                snippet: detail["snippet"].as_str().unwrap_or_default().to_string(),
            });
        }

        Ok(summaries)
    }

    pub async fn list_unread(&self, max_results: u32) -> Result<Vec<EmailSummary>, String> {
        self.search_messages("is:unread", max_results).await
    }

    pub async fn search(&self, query: &str, max_results: u32) -> Result<Vec<EmailSummary>, String> {
        self.search_messages(query, max_results).await
    }

    /*
    Creates a DRAFT response, attached to the message thread
    original.
     */
    pub async fn create_draft_reply(&self, message_id: &str, to: &str, subject: &str, body: &str)
        -> Result<String, String> {
        let access_token = self.get_access_token().await?;
        let client = Client::new();

        let detail_res = client
            .get(format!("https://gmail.googleapis.com/gmail/v1/users/me/messages/{}", message_id))
            .bearer_auth(&access_token)
            .query(&[("format", "metadata"), ("metadataHeaders", "Message-ID")])
            .send()
            .await
            .map_err(|e| format!("Gmail get error: {}", e))?;

        let detail: Value = detail_res.json().await
            .map_err(|e| format!("Gmail get parse error: {}", e))?;

        let thread_id = detail["threadId"].as_str().unwrap_or_default();
        let original_message_id = detail["payload"]["headers"]
            .as_array()
            .and_then(|headers| headers.iter().find(|h| h["name"].as_str() == Some("Message-ID")))
            .and_then(|h| h["value"].as_str())
            .unwrap_or_default();

        let raw_message = format!(
            "To: {}\r\nSubject: {}\r\nIn-Reply-To: {}\r\nReferences: {}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{}",
            to, subject, original_message_id, original_message_id, body
        );
        let encoded = general_purpose::URL_SAFE_NO_PAD.encode(raw_message);

        let res = client
            .post("https://gmail.googleapis.com/gmail/v1/users/me/drafts")
            .bearer_auth(&access_token)
            .json(&json!({
                "message": {
                    "raw": encoded,
                    "threadId": thread_id
                }
            }))
            .send()
            .await
            .map_err(|e| format!("Gmail draft error: {}", e))?;

        if res.status().is_success() {
            Ok("Brouillon créé avec succès.".to_string())
        } else {
            let err_body = res.text().await.unwrap_or_default();
            Err(format!("Échec création brouillon : {}", err_body))
        }
    }

    /*
    Add/remove labels on a message :
    covers read/unread (UNREAD label), favorite (STARRED),
    and archiving (removing the INBOX label).
     */
    pub async fn modify_labels(&self, message_id: &str, add: &[&str], remove: &[&str]) -> Result<String, String> {
        let access_token = self.get_access_token().await?;
        let client = Client::new();

        let res = client
            .post(format!("https://gmail.googleapis.com/gmail/v1/users/me/messages/{}/modify", message_id))
            .bearer_auth(access_token)
            .json(&json!({
                "addLabelIds": add,
                "removeLabelIds": remove
            }))
            .send()
            .await
            .map_err(|e| format!("Gmail modify error: {}", e))?;

        if res.status().is_success() {
            Ok("Modification appliquée.".to_string())
        } else {
            let status = res.status();
            let err_body = res.text().await.unwrap_or_default();
            Err(format!("Échec modification ({}): {}", status, err_body))
        }
    }

    /*
    Move to trash
     */
    pub async fn trash(&self, message_id: &str) -> Result<String, String> {
        let access_token = self.get_access_token().await?;
        let client = Client::new();

        let res = client
            .post(format!("https://gmail.googleapis.com/gmail/v1/users/me/messages/{}/trash", message_id))
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| format!("Gmail trash error: {}", e))?;

        if res.status().is_success() {
            Ok("Email déplacé vers la corbeille.".to_string())
        } else {
            let status = res.status();
            let err_body = res.text().await.unwrap_or_default();
            Err(format!("Échec suppression ({}): {}", status, err_body))
        }
    }
}