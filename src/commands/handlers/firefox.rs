use crate::commands::command::CommandHandler;
use std::process::Command;
use std::env;

/*
Launch the browser
 */
pub struct FirefoxCommand;

impl CommandHandler for FirefoxCommand {
    fn execute(&self) {
        let firefox_path = env::var("FIREFOX_PATH")
            .expect("FIREFOX_PATH is missing in .env");
        Command::new(firefox_path).spawn().expect("Firefox command failed to start");
    }
}