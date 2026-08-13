use std::sync::Arc;
use std::collections::HashSet;
use tokio::sync::Mutex;
use async_trait::async_trait;
use crate::services::gmail::GmailService;
use crate::watchers::watcher::{Watcher, WatcherEvent};

pub struct GmailWatcher {
    gmail: GmailService,
    seen_ids: Arc<Mutex<HashSet<String>>>,
}

impl GmailWatcher {
    pub fn new(gmail: GmailService) -> Self {
        Self { gmail, seen_ids: Arc::new(Mutex::new(HashSet::new())) }
    }
}

#[async_trait]
impl Watcher for GmailWatcher {
    fn name(&self) -> String { String::from("gmail") }

    async fn check(&self) -> Result<Vec<WatcherEvent>, String> {
        let emails = self.gmail.list_unread(5).await?;
        let mut seen = self.seen_ids.lock().await;

        let mut events = Vec::new();
        for email in emails {
            if seen.contains(&email.id) {
                continue;
            }
            seen.insert(email.id.clone());

            let detail = match self.gmail.get_email_details_for_llm(&email.id).await {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[GMAIL WATCHER] Unable to retrieve the details of {}: {}", email.id, e);
                    continue;
                }
            };

            //FIX : no-reply ignore
            let sender = detail.from.to_lowercase();
            if sender.contains("no-reply") || sender.contains("noreply") || sender.contains("newsletter"){
                continue;
            }

            events.push(WatcherEvent {
                source: "gmail".to_string(),
                context: format!(
                    "New important unread email (id: {}). From: {}. Subject: {}. Full content: {}",
                    detail.id, detail.from, detail.subject, detail.body
                ),
            });
        }

        Ok(events)
    }

    fn prompt(&self) -> String {
        "Specific instructions for email management:

        1. INVESTIGATION AND VERIFICATION:
        - Determine the main subject of the message (invitation, appointment, project question, task follow-up, etc.).
        - Use your reference tools (calendar, memory, notes, etc.) to check availability and context before deciding on or preparing a response.

        2. RESPONSE AND SILENCE ('RAS') RULES:
        - If the email is a NEWSLETTER, a PROMOTION (H&M, KFC, etc.), an APP NOTIFICATION (Google, Tiktok, etc.), an AUTOMATED ALERT, or a message requiring NO human response:
        RESPOND ONLY WITH THE WORD: 'RAS'
        - Do NOT generate comments such as 'No action required', 'Promotional email', or 'Draft not necessary'.

        3. ACTION FOR IMPORTANT HUMAN MESSAGES:
        - Prepare a response via 'draft_reply' ONLY if the email comes from a human expecting a genuine reply or action.
        - NEVER use 'draft_reply' for 'no-reply' emails, security alerts, or social media notifications.
        - If a draft is required, enrich it with precise data found via your tools (e.g., your free time slots, project status).
        - Write a summary in French (max. 2 sentences) stating the event and confirming that the draft is ready for validation.".to_string()
    }
}