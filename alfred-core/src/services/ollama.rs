use std::env;
use std::time::Duration;
use serde_json::{json, Value};

pub struct OllamaService;

impl OllamaService {

    /*
    Call ollama serve
     */
    pub async fn chat(messages: &Value, tools: &Value) -> Result<Value, String> {
        /*
        Create body request
         */
        let body = json!({
            "model": "qwen3:4b-instruct",
            "messages": messages,
            "tools": tools,
            "stream": false,
            "options": { "num_ctx": 32000 }
        });

        /*
        Recover ollama api key
         */
        let ollama_api = match env::var("OLLAMA_API") {
            Ok(val) => val,
            Err(_) => return Err("OLLAMA_API URL is missing in env file.".to_string()),
        };

        /*
        Add time out on llm
         */
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))//5 min timeout
            .build()
            .expect("Unable to build the HTTP client");

        /*
        Send request to ollama
         */
        let response = client
            .post(format!("{}/api/chat", ollama_api))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Ollama request error: {}", e))?;

        response.json::<Value>().await
            .map_err(|e| format!("Ollama error during JSON parsing: {}", e))
    }
}