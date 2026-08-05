/*
Quit application
 */
use std::collections::HashMap;
use crate::commands::command::CommandHandler;
use std::process;
use serde_json::{json, Value};
use crate::core::dream::DreamTimer;

pub struct QuitCommand;

impl CommandHandler for QuitCommand {
    fn execute(&self, _: &HashMap<String, String>) -> String{
        String::from("Shutdown in progress...")
    }

    fn description(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "quit",
                "description": "Exit the assistant application.",
                "parameters": {
                    "type": "object",
                    "properties": {}
                }
            }
        })
    }
}