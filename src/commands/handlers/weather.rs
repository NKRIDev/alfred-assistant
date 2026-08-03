use crate::commands::command::CommandHandler;
use crate::services::weather::{WeatherService};

pub struct WeatherCommand;

impl CommandHandler for WeatherCommand {
    fn execute(&self, args: &[String]) -> String{
        if args.len() < 1 {
            return String::from("Missing argument: weather <date: now or YYYY-MM-DD> <city_name>");
        }

        let date_arg = &args[0];
        let city_name = args[1..].join(" "); //retrieve the rest of the string

        /*
        Check if date is 'now' or not
         */
        let date = if date_arg == "now"{
             None
        }
        else {
            Some(date_arg.as_str())
        };

        WeatherService::get_weather_city(&city_name, date)
    }
}