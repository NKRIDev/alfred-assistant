/*
Managing commands: routing and actions
 */
mod time;
mod hello;
mod quit;

use crate::commands::hello::handle_hello;
use crate::commands::time::handle_time;
use crate::commands::quit::handle_quit;

/*
List of the various commands
 */
enum Command{
    Hello,
    Time,
    Quit,
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
        _ => Command::Unkown
    }
}

/*
Executes the action of the requested command
 */
pub fn action_command(input: &str) -> String{
    let command_type: Command = parse_command(input);

    match command_type {
        Command::Hello => handle_hello(),
        Command::Time => handle_time(),
        Command::Quit => handle_quit(),
        Command::Unkown => String::from("Commande inconnue"),
    }
}