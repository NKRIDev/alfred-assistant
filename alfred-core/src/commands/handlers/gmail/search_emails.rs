use std::collections::HashMap;
use serde_json::{json, Value};
use crate::commands::command::CommandHandler;
use crate::services::gmail::GmailService;
use tokio::runtime::Handle;

/*
Search email
 */
pub struct SearchEmailsCommand {
    gmail: GmailService,
}

impl SearchEmailsCommand {
    pub fn new(gmail: GmailService) -> Self { Self { gmail } }
}

impl CommandHandler for SearchEmailsCommand {
    fn execute(&self, args: &HashMap<String, String>) -> String {
        let query = match args.get("query") { Some(v) => v, None => return "Missing argument: query".into() };

        let result = tokio::task::block_in_place(|| {
            Handle::current().block_on(self.gmail.search(query, 10))
        });

        match result {
            Ok(emails) if emails.is_empty() => "Aucun résultat.".to_string(),
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
                "name": "search_emails",
                "description": "Search emails using Gmail search syntax (e.g. 'from:x@y.com', 'subject:invoice', 'after:2026/01/01').",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Gmail search query" }
                    },
                    "required": ["query"]
                }
            }
        })
    }
}