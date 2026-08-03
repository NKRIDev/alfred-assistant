use std::env;
use serde_json::{json, Value};

pub struct OllamaService;

impl OllamaService {

    /*

     */
    pub fn chat(messages: &Value, tools: &Value) -> Value {
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
            Err(_) => return json!({"error": "ollama api url is missing in env file."}),
        };

        /*
        Send request to ollama
         */
        reqwest::blocking::Client::new()
            .post(format!("{}/api/chat", ollama_api))
            .json(&body)
            .send()
            .expect("Erreur requête Ollama")
            .json::<Value>()
            .expect("Erreur parsing réponse Ollama")
    }
}