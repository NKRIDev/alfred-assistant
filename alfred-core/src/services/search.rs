use std::{env, thread};

pub struct SearchService;

impl SearchService {

    /*
    Performs an internet search with the ollama API
     */
    pub fn get_search(search: &str) -> String {
        let search = search.to_string();

        let handle = thread::spawn(move || -> String {
            /*
            Recover ollama api
             */
            let api_key = match env::var("OLLAMA_API_KEY") {
                Ok(val) => val,
                Err(_) => return String::from("OLLAMA_API_KEY is missing in env file."),
            };

            let client = reqwest::blocking::Client::new();
            let body = serde_json::json!({
                "query": search,
                "max_results": 5
            });

            /*
            Create request API
             */
            let response = client
                .post("https://ollama.com/api/web_search")
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&body)
                .send();

            match response {
                Ok(resp) => match resp.text() {
                    Ok(body) => body,
                    Err(e) => return format!("Reading response error : {}", e),
                },
                Err(e) => return format!("Search request error : {}", e),
            }
        });

        match handle.join() {
            Ok(result) => result,
            Err(_) => return String::from("Panic thread."),
        }
    }
}