use std::collections::HashMap;
use serde_json::{json, Value};
use crate::commands::command::CommandHandler;
use crate::services::search::SearchService;

pub struct SearchCommand;

/*
Search on the web with ollama API
 */
impl CommandHandler for SearchCommand {
    fn execute(&self, args: &HashMap<String, String>) -> String {
        let subject = match args.get("query") {
            Some(q) => q,
            None => return String::from("Missing argument: query"),
        };

        SearchService::get_search(subject)
    }

    fn description(&self) -> Value {
        json!({
        "type": "function",
        "function": {
            "name": "search",
            "description": "Search the web for current or recent factual information (news, facts, prices, general knowledge). \
            Do NOT use this to play a song or artist — use manage_music for that instead, even if the request looks like \
            'find me a song' or names a track/artist.",
            "parameters": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "The search query" }
                },
                "required": ["query"]
            }
        }
    })
    }
}