use std::fs;
use serde_json::{json, Value};
use crate::commands::registry::CommandRegistry;
use crate::core::dream::DreamTimer;
use crate::services::memory::MemoryService;
/*
Alfred's central information
 */
pub struct Alfred {
    pub model: String,
    pub system_prompt: String,
    pub registry: CommandRegistry,
    pub history: Vec<Value>,
    pub memory: MemoryService
}

impl Alfred {
    pub fn new(model: String, skill_path: String, database: String, registry: CommandRegistry) -> Self {
        let system_prompt = fs::read_to_string(skill_path)
            .unwrap_or_else(|_| String::from("Tu es Alfred, un assistant utile."));

        /*
        Load database
         */
        let memory = MemoryService::new(&database).expect("Error opening memory database.");

        /*
        Starting the timer for the dream run.
         */
        DreamTimer::start(database, 5);

        /*
        Create Alfred prompt
         */
        let prompt = format!("{}\n\n{}", system_prompt, MemoryService::load_term_memory());

        /*
        Create conversation history
         */
        let system_conv = json!({
            "role": "system",
            "content": prompt,
        });
        let mut history = vec![system_conv];

        /*
        Retrieve past conversations stored in the
        database to provide them as context to the LLM
         */
        if let Ok(events) = memory.get_role_content(){

            for event in &events {
                history.push(json!({ "role": event.0, "content": event.1 }));
            }
        }

        Self{
            model,
            system_prompt,
            registry,
            history,
            memory
        }
    }
}