use std::collections::HashMap;
use serde_json::{json, Value};
use crate::commands::command::CommandHandler;
use crate::services::gmail::GmailService;
use tokio::runtime::Handle;

pub struct ManageEmailCommand {
    gmail: GmailService,
}

impl ManageEmailCommand {
    pub fn new(gmail: GmailService) -> Self { Self { gmail } }
}

impl CommandHandler for ManageEmailCommand {
    fn execute(&self, args: &HashMap<String, String>) -> String {
        let message_id = match args.get("message_id") { Some(v) => v,
            None => return "Missing argument: message_id".into() };
        let action = match args.get("action") { Some(v) => v.as_str(),
            None => return "Missing argument: action".into() };

        let result = tokio::task::block_in_place(|| {
            Handle::current().block_on(async {
                match action {
                    "mark_read" => self.gmail.modify_labels(message_id, &[], &["UNREAD"]).await,
                    "mark_unread" => self.gmail.modify_labels(message_id, &["UNREAD"], &[]).await,
                    "star" => self.gmail.modify_labels(message_id, &["STARRED"], &[]).await,
                    "unstar" => self.gmail.modify_labels(message_id, &[], &["STARRED"]).await,
                    "archive" => self.gmail.modify_labels(message_id, &[], &["INBOX"]).await,
                    "delete" => self.gmail.trash(message_id).await,
                    _ => Err(format!("Action inconnue : {}", action)),
                }
            })
        });

        result.unwrap_or_else(|e| format!("Erreur : {}", e))
    }

    fn description(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "manage_email",
                "description": "Modify an email's status: mark as read/unread, star/unstar, archive (remove from inbox), or delete (move to trash, recoverable for 30 days). Never sends or permanently deletes an email.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "message_id": { "type": "string", "description": "ID of the email (from list_unread_emails or search_emails)" },
                        "action": {
                            "type": "string",
                            "enum": ["mark_read", "mark_unread", "star", "unstar", "archive", "delete"],
                            "description": "Action to perform on the email"
                        }
                    },
                    "required": ["message_id", "action"]
                }
            }
        })
    }
}