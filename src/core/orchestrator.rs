/*
Link between the user, alfred and
the LLM used
 */
use std::collections::HashMap;
use serde_json::{json, Value};
use crate::core::alfred::Alfred;
use crate::services::ollama::OllamaService;

pub struct Orchestrator;

impl Orchestrator {

    pub fn ask_alfred(user_input: &str, alfred: &mut Alfred) -> String {
        /*
        Build conversation
         */
        let tools = alfred.registry.build_tools();

        /*
        Add user question to local conversation history
         */
        let user_content = json!({ "role": "user", "content": user_input });

        /*
        Save user content: in the context and database
         */
        alfred.history.push(user_content);
        alfred.memory.log_event("user", user_input, None).ok();

        loop {
            /*
            Call ollama service
            */
            let response = OllamaService::chat(&Value::Array(alfred.history.clone()), &tools);

            /*
            Save assistant response in the context and databse
             */
            let message = response["message"].clone();
            alfred.history.push(message.clone());

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

                        let tool_result = alfred.registry.execute(name, &args);
                        alfred.history.push(json!({ "role": "tool", "content": tool_result }));
                        alfred.memory.log_event("tool", &tool_result, Some(name)).ok();
                    }
                }
                _ => {
                    let reply = message["content"].as_str().unwrap_or("Pas de réponse.");
                    alfred.memory.log_event("assistant", &reply, None).ok();
                    return reply.to_string();
                }
            }
        }
    }
}