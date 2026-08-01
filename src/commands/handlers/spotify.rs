use crate::commands::command::CommandHandler;
use std::process::Command;
use std::env;

/*
Launch spotify app
 */
pub struct SpotifyCommand;

impl CommandHandler for SpotifyCommand {
    fn execute(&self) -> String{
        let spotify_path = env::var("SPOTIFY_PATH")
            .expect("SPOTIFY_PATH is missing in .env");

        match Command::new(spotify_path).spawn() {
            Ok(output) => {String::from("Spotify started successfully !")},
            Err(_) => {String::from("Spotify command failed to start.")}
        }
    }
}