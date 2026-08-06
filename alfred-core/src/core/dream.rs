use crate::services::memory::MemoryService;
use crate::services::ollama::OllamaService;

/*
Task that updates the assistant's memory
 */
pub struct DreamTimer;

impl DreamTimer {

    /*
    Triggers a single dream consolidation task asynchronously without blocking
     */
    pub async fn trigger(database: String) {
        println!("[SYSTEM] Cleaning and saving memory...");
        /*
        Generate prompt
        */
        let prompt_dream = {
            let service = MemoryService::new(&database).ok();
            service.and_then(|s| s.prepared_dream_prompt().ok().flatten())
        };

        /*
        Call ollama with async network
        */
        if let Some(prompt) = prompt_dream {
            println!("[SYSTEM] Auto Dream process started.");

            //Message and call ollama API
            let messages = serde_json::json!([
                {"role": "user", "content": prompt},
            ]);

            match OllamaService::chat(&messages, &serde_json::json!([])).await {
                Ok(response) => {
                    /*
                    FIX : we require real non-empty text content
                    before writing anything
                     */
                    match response["message"]["content"].as_str() {
                        Some(summary) if !summary.trim().is_empty() => {
                            if let Ok(service) = MemoryService::new(&database) {
                                match service.finalization_dream(summary) {
                                    Ok(msg) => println!("[SYSTEM] {}", msg),
                                    Err(e) => eprintln!("[DREAM ERROR] {}", e),
                                }
                            }
                        }
                        _ => {
                            eprintln!(
                                "[DREAM ERROR] Réponse Ollama sans contenu exploitable, \
                                memory.md conservé tel quel. Events non consolidés, \
                                ils seront repris au prochain dream."
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[DREAM ERROR] Échec de l'appel Ollama : {}. memory.md \
                    conservé tel quel. Events non consolidés.", e);
                }
            }
        }
        else {
            println!("[SYSTEM] No events to consolidate.");
        }
    }
}