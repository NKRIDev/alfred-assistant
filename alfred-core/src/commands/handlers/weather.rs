use std::collections::HashMap;
use serde_json::{json, Value};
use crate::commands::command::CommandHandler;
use crate::services::weather::{WeatherService};

pub struct WeatherCommand;

impl CommandHandler for WeatherCommand {
    fn execute(&self, args: &HashMap<String, String>) -> String{
        let date_arg = args.get("date").map(|s| s.as_str()).unwrap_or("now");
        let city_name = match args.get("city") {
            Some(c) => c,
            None => return String::from("Missing argument: city"),
        };

        /*
        Check if date is 'now' or not
         */
        let date = if date_arg == "now"{
             None
        }
        else {
            Some(date_arg)
        };

        WeatherService::get_weather_city(&city_name, date)
    }

    fn description(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "weather",
                "description": "Get the current or forecasted weather for a given city.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "date": {
                            "type": "string",
                            "description": "Date in YYYY-MM-DD format, or 'now' for current weather"
                        },
                        "city": {
                            "type": "string",
                            "description": "Name of the city"
                        }
                    },
                    "required": ["date", "city"]
                }
            }
        })
    }
}