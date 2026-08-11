/*
Link between the user, alfred and
the LLM used
 */
use std::collections::HashMap;
use serde_json::{json, Value};
use crate::core::alfred::Alfred;
use crate::services::mistral::MistralService;
use crate::services::ollama::OllamaService;

pub struct Orchestrator;

impl Orchestrator {

    pub async fn ask_alfred(user_input: &str, alfred: &mut Alfred) -> String {
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
            //FIX : clear null object
            alfred.history.retain(|m| !m.is_null() && m.is_object());

            /*
            Call ollama service
            */
            let response = match OllamaService::chat(&Value::Array(alfred.history.clone()), &tools).await {
                Ok(res) => res,
                Err(err) => {
                    eprintln!("[ERROR] Ollama request failed: {}", err);
                    return format!("An error occurred with the connection to the LLM : {}", err);
                }
            };

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

                    /*
                    Apply context opti.
                     */
                    alfred.trim_history(20);

                    return reply.to_string();
                }
            }
        }
    }

    pub async fn ask_alfred_mistral(user_input: &str, alfred: &mut Alfred) -> String {
        let tools = alfred.registry.build_tools();

        let user_content = json!({ "role": "user", "content": user_input });

        alfred.history.push(user_content);
        alfred.memory.log_event("user", user_input, None).ok();

        loop {
            //FIX : clear null object
            alfred.history.retain(|m| !m.is_null() && m.is_object());
            
            let response = match MistralService::chat(&Value::Array(alfred.history.clone()), &tools).await {
                Ok(res) => res,
                Err(err) => {
                    eprintln!("[ERROR] Mistral API request failed: {}", err);
                    return format!("Erreur lors de la connexion à l'API Mistral : {}", err);
                }
            };

            let message = response["message"].clone();
            alfred.history.push(message.clone());

            match message["tool_calls"].as_array() {
                Some(calls) if !calls.is_empty() => {
                    for call in calls {
                        let name = call["function"]["name"].as_str().unwrap_or("");
                        let tool_call_id = call["id"].as_str().unwrap_or("");

                        let args_raw = call["function"]["arguments"].as_str().unwrap_or("{}");
                        let args_json: Value = serde_json::from_str(args_raw).unwrap_or(json!({}));

                        let args: HashMap<String, String> = args_json
                            .as_object()
                            .map(|obj| {
                                obj.iter()
                                    .map(|(k, v)| {
                                        let val_str = match v {
                                            Value::String(s) => s.clone(),
                                            _ => v.to_string(),
                                        };
                                        (k.clone(), val_str)
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();

                        let tool_result = alfred.registry.execute(name, &args);

                        alfred.history.push(json!({
                            "role": "tool",
                            "name": name,
                            "tool_call_id": tool_call_id,
                            "content": tool_result
                        }));

                        alfred.memory.log_event("tool", &tool_result, Some(name)).ok();
                    }
                }
                _ => {
                    let reply = message["content"].as_str().unwrap_or("Pas de réponse.");
                    let reply_edit = strip_markdown(reply);
                    alfred.memory.log_event("assistant", &reply_edit, None).ok();

                    alfred.trim_history(20);

                    return reply_edit.to_string();
                }
            }
        }
    }
}

pub fn strip_markdown(text: &str) -> String {
    let mut result = text.to_string();
    result = regex::Regex::new(r"\*\*(.+?)\*\*").unwrap().replace_all(&result, "$1").to_string();
    result = regex::Regex::new(r"\*(.+?)\*").unwrap().replace_all(&result, "$1").to_string();
    result = regex::Regex::new(r"(?m)^\d+\.\s+").unwrap().replace_all(&result, "").to_string();
    result = regex::Regex::new(r"(?m)^[-*]\s+").unwrap().replace_all(&result, "").to_string();
    result = regex::Regex::new(r"(?m)^#+\s+").unwrap().replace_all(&result, "").to_string();
    result = regex::Regex::new(r"\[(.+?)\]\(.+?\)").unwrap().replace_all(&result, "$1").to_string();
    result
}