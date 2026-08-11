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
        "Email-specific instructions:
        1. INVESTIGATION AND CROSS-REFERENCING:
        - Determine the main subject of the message (appointment, project-related question, task follow-up, document request, etc.).
        - Feel free to use your reference tools (calendar, notes search, memory, tasks, etc.) to verify the actual context before making
         a decision or drafting a response.

        2. RESPONSE AND ACTION RULES:
        - Use 'draft_reply' ONLY if the email comes from a human sender expecting a genuine response or action from you.
        - NEVER reply to automated notifications, 'no-reply' emails, security alerts, banking communications, newsletters, or social media messages.
        - If a draft is required, enrich it with precise data found using your tools (e.g., your available time slots, project status, etc.).".to_string()
    }
}