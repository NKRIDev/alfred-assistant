use crate::commands::command::CommandHandler;
use std::process::Command;
use std::env;

/*
Launch the browser
 */
pub struct FirefoxCommand;

impl CommandHandler for FirefoxCommand {
    fn execute(&self)  -> String{
        let firefox_path = env::var("FIREFOX_PATH")
            .expect("FIREFOX_PATH is missing in .env");

        match Command::new(firefox_path).spawn() {
            Ok(mut child) => {String::from("Firefox started successfully !")},
            Err(_) => {String::from("Failed to start firefox.")}
        }
    }
}