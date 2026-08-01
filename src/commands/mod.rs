/*
Managing commands: routing and actions
 */
mod command;
mod handlers;

use crate::commands::command::CommandHandler;
use crate::commands::handlers::{
    hello::HelloCommand,
    time::TimeCommand,
    quit::QuitCommand,
    firefox::FirefoxCommand,
};
use crate::commands::handlers::spotify::SpotifyCommand;
/*
List of the various commands
 */
enum Command{
    Hello,
    Time,
    Quit,
    Firefox,
    Spotify,
    Unkown,
}

/*
Returns a command based on user input
 */
fn parse_command(input: &str) -> Command {
    match input{
        "hello" | "bonjour" => Command::Hello,
        "quit" => Command::Quit,
        "heure" | "time" => Command::Time,
        "firefox" => Command::Firefox,
        "spotify" | "music" => Command::Spotify,
        _ => Command::Unkown
    }
}

/*
Executes the action of the requested command
 */
pub fn action_command(input: &str) {
    let command_type: Command = parse_command(input);

    match command_type {
        Command::Hello => {
            HelloCommand.execute();
        },
        Command::Time => {
            TimeCommand.execute();
        },
        Command::Quit => {
            QuitCommand.execute();
        },
        Command::Firefox => {
            FirefoxCommand.execute();
        }
        Command::Spotify => {
            SpotifyCommand.execute();
        }
        Command::Unkown => { println!("Command inconnue."); },
    }
}