/*
Link between the user, alfred and
the LLM used
 */
use std::collections::HashMap;
use serde_json::{json, Value};
use crate::commands::registry::CommandRegistry;
use crate::services::ollama::OllamaService;

pub struct Orchestrator;

impl Orchestrator {

    pub fn ask_alfred(user_input: &str, registry: &CommandRegistry, system_prompt: &str) -> String {
        /*
        Build conversation
         */
        let tools = registry.build_tools();
        let mut messages = vec![
            json!({ "role": "system", "content": system_prompt }), //Alfred rules+system
            json!({ "role": "user", "content": user_input }), //User input, question, words etc.
        ];

        loop {
            /*
            Call ollama service
            */
            let response = OllamaService::chat(&Value::Array(messages.clone()), &tools);

            /*
            Add response to history
             */
            let message = response["message"].clone();
            messages.push(message.clone());

            /*
            Check if LLM need tools
             */
            match message["tool_calls"].as_array() {
                Some(calls) if !calls.is_empty() => {
                    //Execute tool
                    for call in calls {
                        let name = call["function"]["name"].as_str().unwrap_or("");

                        let args: HashMap<String, String> = call["function"]["arguments"]
                            .as_object()
                            .map(|obj| {
                                obj.iter()
                                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                                    .collect()
                            })
                            .unwrap_or_default();

                        let tool_result = registry.execute(name, &args);
                        messages.push(json!({ "role": "tool", "content": tool_result }));
                    }
                }
                _ => {
                    return message["content"].as_str().unwrap_or("Pas de réponse.").to_string();
                }
            }
        }
    }
}