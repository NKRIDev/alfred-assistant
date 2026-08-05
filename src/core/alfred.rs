use std::fs;
use serde_json::{json, Value};
use crate::commands::registry::CommandRegistry;
use crate::core::dream::DreamTimer;
use crate::services::memory;
use crate::services::memory::MemoryService;
/*
Alfred's central information
 */
pub struct Alfred {
    pub model: String,
    pub system_prompt: String,
    pub registry: CommandRegistry,
    pub history: Vec<Value>,
    pub memory: MemoryService,
    pub database_url: String,
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
        let history = vec![system_conv];

        Self{
            model,
            system_prompt,
            registry,
            history,
            memory,
            database_url: database.clone(),
        }
    }

    /*
    So that the assistant is not a huge context, we keep the
    system context + the last 'max_messages' messages
     */
    pub fn trim_history(&mut self, max_messages: usize) {
        if self.history.len() <= max_messages + 1 {
            return;
        }

        let system = self.history[0].clone();
        let tail_start = self.history.len() - max_messages;
        let mut trimmed = vec![system];
        trimmed.extend_from_slice(&self.history[tail_start..]);
        self.history = trimmed;
    }
}