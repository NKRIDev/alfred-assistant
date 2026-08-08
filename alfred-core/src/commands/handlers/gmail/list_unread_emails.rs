use std::collections::HashMap;
use serde_json::{json, Value};
use crate::commands::command::CommandHandler;
use crate::services::gmail::GmailService;
use tokio::runtime::Handle;

/*
Display unread emails
 */
pub struct ListUnreadEmailsCommand {
    gmail: GmailService,
}

impl ListUnreadEmailsCommand {
    pub fn new(gmail: GmailService) -> Self { Self { gmail } }
}

impl CommandHandler for ListUnreadEmailsCommand {
    fn execute(&self, _args: &HashMap<String, String>) -> String {
        let result = tokio::task::block_in_place(|| {
            Handle::current().block_on(self.gmail.list_unread(10))
        });

        match result {
            Ok(emails) if emails.is_empty() => "Aucun email non lu.".to_string(),
            Ok(emails) => emails.iter()
                .map(|e| format!("[{}] De: {} — Sujet: {} — {}", e.id, e.from, e.subject, e.snippet))
                .collect::<Vec<_>>()
                .join("\n"),
            Err(e) => {
                eprintln!("[GMAIL ERROR] {}", e);
                format!("Erreur : {}", e)
            },
        }
    }

    fn description(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "list_unread_emails",
                "description": "List unread emails in the inbox.",
                "parameters": { "type": "object", "properties": {} }
            }
        })
    }
}