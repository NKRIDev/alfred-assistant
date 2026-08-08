use std::collections::HashMap;
use serde_json::{json, Value};
use crate::commands::command::CommandHandler;
use crate::services::calendar::CalendarService;
use tokio::runtime::Handle;

pub struct ListEventsCommand { calendar: CalendarService }
impl ListEventsCommand {
    pub fn new(calendar: CalendarService) -> Self { Self { calendar } }
}
impl CommandHandler for ListEventsCommand {
    fn execute(&self, _args: &HashMap<String, String>) -> String {
        let result = tokio::task::block_in_place(|| {
            Handle::current().block_on(self.calendar.list_upcoming(10))
        });
        match result {
            Ok(events) if events.is_empty() => "Aucun événement à venir.".to_string(),
            Ok(events) => events.iter()
                .map(|e| format!("[{}] {} — de {} à {}", e.id, e.summary, e.start, e.end))
                .collect::<Vec<_>>().join("\n"),
            Err(e) => format!("Erreur : {}", e),
        }
    }
    fn description(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "list_events",
                "description": "List upcoming calendar events.",
                "parameters": { "type": "object", "properties": {} }
            }
        })
    }
}