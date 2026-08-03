use crate::commands::command::CommandHandler;
use crate::services::search::SearchService;
use crate::services::weather::{WeatherService};

pub struct SearchCommand;

/*
Search on the web with ollama API
 */
impl CommandHandler for SearchCommand {
    fn execute(&self, args: &[String]) -> String {
        if args.len() < 1 {
            return String::from("Missing argument : specify the search to be performed")
        }

        let search_value = args.join(" ");
        SearchService::get_search(search_value)
    }
}