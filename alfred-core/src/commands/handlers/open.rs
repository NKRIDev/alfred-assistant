/*
Launch an application
 */
use std::collections::HashMap;
use serde_json::{json, Value};
use crate::commands::command::CommandHandler;
use crate::services::application::ApplicationService;

pub struct OpenCommand {
    app_service: ApplicationService,
}

impl OpenCommand {
    pub fn new(app_service: ApplicationService) -> OpenCommand {
        OpenCommand { app_service }
    }
}

impl CommandHandler for OpenCommand {
    fn execute(&self, args: &HashMap<String, String>) -> String{
        let app_name = match args.get("app_name") {
            Some(a) => a,
            None => return String::from("Missing argument: app_name"),
        };

        self.app_service.open(app_name)
    }

    fn description(&self) -> Value {
        json!({
        "type": "function",
        "function": {
            "name": "open",
            "description": "Open an application on the computer with no specific content .\
            (e.g. open a text editor, open a browser). Do NOT use this to play music or a specific \
            song — use manage_music for that instead.",
            "parameters": {
                "type": "object",
                "properties": {
                    "app_name": {
                        "type": "string",
                        "description": "Name of the application to open"
                    }
                },
                "required": ["app_name"]
            }
        }
    })
    }
}