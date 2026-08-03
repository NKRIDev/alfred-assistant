use std::fs;
use crate::commands::registry::CommandRegistry;

/*
Alfred's central information
 */
pub struct Alfred {
    pub model: String,
    pub system_prompt: String,
    pub registry: CommandRegistry
}

impl Alfred {
    pub fn new(model: String, skill_path: String, registry: CommandRegistry) -> Self {
        let system_prompt = fs::read_to_string(skill_path)
            .unwrap_or_else(|_| String::from("Tu es Alfred, un assistant utile."));

        Self{
            model,
            system_prompt,
            registry,
        }
    }
}