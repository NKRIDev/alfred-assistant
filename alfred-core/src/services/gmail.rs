use reqwest::Client;
use serde_json::{json, Value};
use base64::{engine::general_purpose, Engine};
use crate::services::google_auth::GoogleAuthService;
/*
Service to communicate with the Google gmail service
 */
#[derive(Clone)]
pub struct GmailService {
    auth: GoogleAuthService,
}

#[derive(serde::Serialize)]
pub struct EmailSummary {
    pub id: String,
    pub thread_id: String,
    pub from: String,
    pub subject: String,
    pub snippet: String,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct EmailDetail {
    pub id: String,
    pub thread_id: String,
    pub from: String,
    pub subject: String,
    pub body: String,
}

impl GmailService {
    pub fn new(auth: GoogleAuthService) -> Self {
        Self { auth }
    }

    /*
    Searches for messages based on a Gmail query (search syntax
    Native Gmail: "is:unread", "from:x@y.com", "subject:invoice", etc.)
    then get the metadata (From, Subject) + snippet for each.
     */
    async fn search_messages(&self, query: &str, max_results: u32) -> Result<Vec<EmailSummary>, String> {
        let access_token = self.auth.get_access_token().await?;
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

    pub async fn list_important_unread(&self, max_results: u32) -> Result<Vec<EmailSummary>, String> {
        let query = "is:unread is:important in:inbox";
        self.search_messages(query, max_results).await
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
        let access_token = self.auth.get_access_token().await?;
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
        let access_token = self.auth.get_access_token().await?;
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
        let access_token = self.auth.get_access_token().await?;
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

    /*
    Retrieves the raw text content of an email to pass it to the LLM
     */
    pub async fn get_email_details_for_llm(&self, message_id: &str) -> Result<EmailDetail, String> {
        let access_token = self.auth.get_access_token().await?;
        let client = Client::new();

        //Retrieval of the email in "full" format to access the complete body of the message
        let detail_res = client
            .get(format!("https://gmail.googleapis.com/gmail/v1/users/me/messages/{}", message_id))
            .bearer_auth(&access_token)
            .query(&[("format", "full")])
            .send()
            .await
            .map_err(|e| format!("Gmail GET request error: {}", e))?;

        if !detail_res.status().is_success() {
            let status = detail_res.status();
            let err_body = detail_res.text().await.unwrap_or_default();
            return Err(format!("Email retrieval failed({}) : {}", status, err_body));
        }

        let detail: Value = detail_res.json().await
            .map_err(|e| format!("Erreur parsing JSON Gmail detail: {}", e))?;

        //Extraction of headers (From, Subject)
        let headers = detail["payload"]["headers"].as_array().cloned().unwrap_or_default();
        let get_header = |name: &str| -> String {
            headers.iter()
                .find(|h| h["name"].as_str().map(|s| s.to_lowercase()) == Some(name.to_lowercase()))
                .and_then(|h| h["value"].as_str())
                .unwrap_or("(inconnu)")
                .to_string()
        };

        //Extract body as plain text
        let body = self.extract_plain_text_body(&detail["payload"]);

        Ok(EmailDetail {
            id: detail["id"].as_str().unwrap_or_default().to_string(),
            thread_id: detail["threadId"].as_str().unwrap_or_default().to_string(),
            from: get_header("From"),
            subject: get_header("Subject"),
            body,
        })
    }

    /*
    Helper to traverse the MIME tree and extract the raw text
     */
    fn extract_plain_text_body(&self, payload: &Value) -> String {
        // 1. Tenter d'extraire le texte directement à la racine
        if let Some(body_data) = payload["body"]["data"].as_str() {
            if let Ok(decoded) = general_purpose::URL_SAFE_NO_PAD.decode(body_data)
                .or_else(|_| general_purpose::URL_SAFE.decode(body_data)) {
                let text = String::from_utf8_lossy(&decoded).to_string();
                if !text.trim().is_empty() {
                    return text;
                }
            }
        }

        // 2. Si le message est multipart, parcourir les 'parts'
        if let Some(parts) = payload["parts"].as_array() {
            // Passe 1 : Recherche prioritaire du text/plain
            for part in parts {
                let mime_type = part["mimeType"].as_str().unwrap_or_default();
                if mime_type == "text/plain" {
                    if let Some(data) = part["body"]["data"].as_str() {
                        if let Ok(decoded) = general_purpose::URL_SAFE_NO_PAD.decode(data)
                            .or_else(|_| general_purpose::URL_SAFE.decode(data)) {
                            let text = String::from_utf8_lossy(&decoded).to_string();
                            if !text.trim().is_empty() {
                                return text;
                            }
                        }
                    }
                }

                // Récursivité si sous-parties imbriquées
                if part["parts"].is_array() {
                    let nested = self.extract_plain_text_body(part);
                    if !nested.is_empty() && nested != "(Contenu illisible)" {
                        return nested;
                    }
                }
            }

            // Passe 2 : Fallback sur text/html si aucun text/plain n'a été trouvé
            for part in parts {
                let mime_type = part["mimeType"].as_str().unwrap_or_default();
                if mime_type == "text/html" {
                    if let Some(data) = part["body"]["data"].as_str() {
                        if let Ok(decoded) = general_purpose::URL_SAFE_NO_PAD.decode(data)
                            .or_else(|_| general_purpose::URL_SAFE.decode(data)) {
                            let html_text = String::from_utf8_lossy(&decoded).to_string();
                            // Nettoyage sommaire des balises HTML
                            let clean_text = html_text
                                .replace("<br>", "\n")
                                .replace("<br/>", "\n")
                                .replace("</p>", "\n");
                            // Supprimer le reste des balises HTML avec du regex basique/remplacement
                            let plain = clean_text.chars().fold((String::new(), false), |(mut acc, inside), c| {
                                match c {
                                    '<' => (acc, true),
                                    '>' => (acc, false),
                                    _ if !inside => { acc.push(c); (acc, false) },
                                    _ => (acc, true),
                                }
                            }).0;

                            if !plain.trim().is_empty() {
                                return plain;
                            }
                        }
                    }
                }
            }
        }

        // 3. Dernier fallback sur le snippet de l'e-mail
        if let Some(snippet) = payload["snippet"].as_str() {
            if !snippet.trim().is_empty() {
                return snippet.to_string();
            }
        }

        "(Contenu illisible)".to_string()
    }
}