use std::collections::HashMap;
use serde_json::{json, Value};
use crate::commands::command::CommandHandler;
use crate::services::calendar::CalendarService;
use tokio::runtime::Handle;

pub struct UpdateEventCommand { calendar: CalendarService }
impl UpdateEventCommand {
    pub fn new(calendar: CalendarService) -> Self { Self { calendar } }
}
impl CommandHandler for UpdateEventCommand {
    fn execute(&self, args: &HashMap<String, String>) -> String {
        let event_id = match args.get("event_id") { Some(v) => v, None => return "Missing argument: event_id".into() };
        let summary = args.get("summary").map(|s| s.as_str());
        let start = args.get("start").map(|s| s.as_str());
        let end = args.get("end").map(|s| s.as_str());

        let result = tokio::task::block_in_place(|| {
            Handle::current().block_on(self.calendar.update_event(event_id, summary, start, end))
        });
        result.unwrap_or_else(|e| format!("Erreur : {}", e))
    }
    fn description(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "update_event",
                "description": "Update an existing calendar event. Only provided fields are changed.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "event_id": { "type": "string", "description": "ID of the event (from list_events)" },
                        "summary": { "type": "string", "description": "New title, optional" },
                        "start": { "type": "string", "description": "New start datetime RFC3339, optional" },
                        "end": { "type": "string", "description": "New end datetime RFC3339, optional" }
                    },
                    "required": ["event_id"]
                }
            }
        })
    }
}