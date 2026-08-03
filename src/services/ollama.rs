use std::env;
use std::time::Duration;
use serde_json::{json, Value};

pub struct OllamaService;

impl OllamaService {

    /*
    Call ollama serve
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
        Add time out on llm
         */
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(300))//5 min timeout
            .build()
            .expect("Unable to build the HTTP client");

        /*
        Send request to ollama
         */
        client.post(format!("{}/api/chat", ollama_api))
            .json(&body)
            .send()
            .expect("Ollama request error")
            .json::<Value>()
            .expect("Ollama error during json parsing")
    }
}