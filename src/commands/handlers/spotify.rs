use crate::commands::command::CommandHandler;
use std::process::Command;
use std::env;

/*
Launch spotify app
 */
pub struct SpotifyCommand;

impl CommandHandler for SpotifyCommand {
    fn execute(&self) {
        let spotify_path = env::var("SPOTIFY_PATH")
            .expect("SPOTIFY_PATH is missing in .env");
        Command::new(spotify_path).spawn().expect("Spotify command failed to start");
    }
}