use std::collections::HashMap;
use serde_json::{json, Value};
use crate::commands::command::CommandHandler;
use crate::services::gmail::GmailService;
use tokio::runtime::Handle;

/*
Writes a draft reply to the message
 */
pub struct DraftReplyCommand {
    gmail: GmailService,
}

impl DraftReplyCommand {
    pub fn new(gmail: GmailService) -> Self { Self { gmail } }
}

impl CommandHandler for DraftReplyCommand {
    fn execute(&self, args: &HashMap<String, String>) -> String {
        let message_id = match args.get("message_id") { Some(v) => v, None => return "Missing argument: message_id".into() };
        let to = match args.get("to") { Some(v) => v, None => return "Missing argument: to".into() };
        let subject = match args.get("subject") { Some(v) => v, None => return "Missing argument: subject".into() };
        let body = match args.get("body") { Some(v) => v, None => return "Missing argument: body".into() };

        let result = tokio::task::block_in_place(|| {
            Handle::current().block_on(self.gmail.create_draft_reply(message_id, to, subject, body))
        });

        result.unwrap_or_else(|e| format!("Erreur : {}", e))
    }

    fn description(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "draft_reply",
                "description": "Create a DRAFT reply to an email. This NEVER sends the email — it only saves a draft for the user to review and send manually.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "message_id": { "type": "string", "description": "ID of the email to reply to (from list_unread_emails or search_emails)" },
                        "to": { "type": "string", "description": "Recipient email address" },
                        "subject": { "type": "string", "description": "Reply subject" },
                        "body": { "type": "string", "description": "Reply body content" }
                    },
                    "required": ["message_id", "to", "subject", "body"]
                }
            }
        })
    }
}