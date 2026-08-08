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
        // 1. Récupération des outils enregistrés dans Alfred
        let tools = alfred.registry.build_tools();

        // 2. Formatage du message utilisateur
        let user_content = json!({ "role": "user", "content": user_input });

        // 3. Sauvegarde dans l'historique local et en mémoire
        alfred.history.push(user_content);
        alfred.memory.log_event("user", user_input, None).ok();

        loop {
            // 4. Appel au service Mistral
            let response = match MistralService::chat(&Value::Array(alfred.history.clone()), &tools).await {
                Ok(res) => res,
                Err(err) => {
                    eprintln!("[ERROR] Mistral API request failed: {}", err);
                    return format!("Erreur lors de la connexion à l'API Mistral : {}", err);
                }
            };

            // 5. Extraction et sauvegarde de la réponse de l'assistant
            let message = response["message"].clone();
            alfred.history.push(message.clone());

            // 6. Vérification si le modèle souhaite exécuter des outils
            match message["tool_calls"].as_array() {
                Some(calls) if !calls.is_empty() => {
                    for call in calls {
                        let name = call["function"]["name"].as_str().unwrap_or("");
                        let tool_call_id = call["id"].as_str().unwrap_or("");

                        // L'API Mistral renvoie les arguments sous forme de chaîne JSON (ex: "{\"city\": \"Paris\"}")
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

                        // Exécution de l'outil via Alfred
                        let tool_result = alfred.registry.execute(name, &args);

                        // Enregistrement du résultat au format attendu par Mistral (avec tool_call_id)
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
                    // Si aucun outil n'est appelé, on récupère le texte final
                    let reply = message["content"].as_str().unwrap_or("Pas de réponse.");
                    alfred.memory.log_event("assistant", reply, None).ok();

                    // Nettoyage de l'historique si nécessaire
                    alfred.trim_history(20);

                    return reply.to_string();
                }
            }
        }
    }
}