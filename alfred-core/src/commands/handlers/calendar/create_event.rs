use std::collections::HashMap;
use serde_json::{json, Value};
use crate::commands::command::CommandHandler;
use crate::services::calendar::CalendarService;
use tokio::runtime::Handle;

pub struct CreateEventCommand { calendar: CalendarService }
impl CreateEventCommand {
    pub fn new(calendar: CalendarService) -> Self { Self { calendar } }
}
impl CommandHandler for CreateEventCommand {
    fn execute(&self, args: &HashMap<String, String>) -> String {
        let summary = match args.get("summary") { Some(v) => v, None => return "Missing argument: summary".into() };
        let start = match args.get("start") { Some(v) => v, None => return "Missing argument: start".into() };
        let end = match args.get("end") { Some(v) => v, None => return "Missing argument: end".into() };
        let description = args.get("description").map(|s| s.as_str()).unwrap_or("");

        let result = tokio::task::block_in_place(|| {
            Handle::current().block_on(self.calendar.create_event(summary, start, end, description))
        });
        result.unwrap_or_else(|e| format!("Erreur : {}", e))
    }
    fn description(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "create_event",
                "description": "Create a calendar event. Dates must be in RFC3339 format (e.g. 2026-08-15T14:00:00+02:00).",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "summary": { "type": "string", "description": "Event title" },
                        "start": { "type": "string", "description": "Start datetime, RFC3339 format" },
                        "end": { "type": "string", "description": "End datetime, RFC3339 format" },
                        "description": { "type": "string", "description": "Optional event description" }
                    },
                    "required": ["summary", "start", "end"]
                }
            }
        })
    }
}