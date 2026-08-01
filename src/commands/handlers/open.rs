/*
Launch an application
 */
use std::env;
use std::process::Command;
use crate::commands::command::CommandHandler;

pub struct OpenCommand;

impl CommandHandler for OpenCommand {
    fn execute(&self, args: &[String]) -> String{
        if args.len() < 1 {
            return String::from("Missing argument; specify the name of the app to launch.");
        }

        match args[0].as_str() {
            "firefox" => {
                let firefox_path = env::var("FIREFOX_PATH")
                    .expect("FIREFOX_PATH is missing in .env");

                match Command::new(firefox_path).spawn() {
                    Ok(mut child) => {String::from("Firefox started successfully !")},
                    Err(_) => {String::from("Failed to start firefox.")}
                }
            },
            "spotif" => {
                let spotify_path = env::var("SPOTIFY_PATH")
                    .expect("SPOTIFY_PATH is missing in .env");

                match Command::new(spotify_path).spawn() {
                    Ok(output) => {String::from("Spotify started successfully !")},
                    Err(_) => {String::from("Spotify command failed to start.")}
                }
            },
            _ => String::from("The application was not found.")
        }

    }
}