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

            if let Ok(response) = OllamaService::chat(&messages, &serde_json::json!([])).await {
                let summary = response["message"]["content"]
                    .as_str()
                    .unwrap_or("Error: no summary generated")
                    .to_string();

                //Sync with data base
                if let Ok(service) = MemoryService::new(&database) {
                    match service.finalization_dream(&summary) {
                        Ok(msg) => println!("[SYSTEM] {}", msg),
                        Err(e) => eprintln!("[DREAM ERROR] {}", e),
                    }
                }
            }
        } else {
            println!("[SYSTEM] No events to consolidate.");
        }
    }
}