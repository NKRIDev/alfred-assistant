/*
Recover paris time and display this
 */
use std::collections::HashMap;
use chrono::Utc;
use chrono_tz::Europe::Paris;
use serde_json::{json, Value};
use crate::commands::command::CommandHandler;

pub struct TimeCommand;

impl CommandHandler for TimeCommand {
    fn execute(&self, _: &HashMap<String, String>) -> String{
        let paris_time = Utc::now().with_timezone(&Paris);
        paris_time.format("%H:%M:%S").to_string()
    }

    fn description(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "time",
                "description": "Get the current time in Paris.",
                "parameters": {
                    "type": "object",
                    "properties": {}
                }
            }
        })
    }
}