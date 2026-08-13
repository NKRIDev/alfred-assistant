use std::collections::HashMap;
use serde_json::{json, Value};
use crate::commands::command::CommandHandler;
use crate::services::morning::MorningBriefingService;

pub struct BriefingCommand {
    briefing: MorningBriefingService,
}

/*
Wake up command
 */
impl BriefingCommand {
    pub fn new(briefing: MorningBriefingService) -> Self {
        Self { briefing }
    }
}

impl CommandHandler for BriefingCommand {
    fn execute(&self, args: &HashMap<String, String>) -> String {
        let briefing_service = self.briefing.clone();

        //Dont block the main thread
        let result = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                briefing_service.generate_briefing().await
            })
        }).join();

        match result {
            Ok(Ok(prompt_data)) => prompt_data,
            Ok(Err(err)) => format!("Error generating the briefing: {}", err),
            Err(_) => String::from("Thread failure during briefing execution."),
        }
    }

    fn description(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "morning_briefing",
                "description": "Generates and retrieves the full morning briefing (day's agenda, upcoming appointments, and important messages). \
                To be used when the user asks for their wake-up call, agenda for the day, or briefing.",
                "parameters": {
                    "type": "object",
                    "properties": {}
                }
            }
        })
    }
}