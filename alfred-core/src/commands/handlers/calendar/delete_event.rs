use std::collections::HashMap;
use serde_json::{json, Value};
use crate::commands::command::CommandHandler;
use crate::services::calendar::CalendarService;
use tokio::runtime::Handle;

/*
Calendar delete
 */
pub struct DeleteEventCommand { calendar: CalendarService }
impl DeleteEventCommand {
    pub fn new(calendar: CalendarService) -> Self { Self { calendar } }
}
impl CommandHandler for DeleteEventCommand {
    fn execute(&self, args: &HashMap<String, String>) -> String {
        let event_id = match args.get("event_id") { Some(v) => v, None => return "Missing argument: event_id".into() };
        let result = tokio::task::block_in_place(|| {
            Handle::current().block_on(self.calendar.delete_event(event_id))
        });
        result.unwrap_or_else(|e| format!("Erreur : {}", e))
    }
    fn description(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "delete_event",
                "description": "Permanently delete a calendar event. This action is irreversible.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "event_id": { "type": "string", "description": "ID of the event to delete (from list_events)" }
                    },
                    "required": ["event_id"]
                }
            }
        })
    }
}