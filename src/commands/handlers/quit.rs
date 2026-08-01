/*
Quit application
 */
use crate::commands::command::CommandHandler;
use std::process;

pub struct QuitCommand;

impl CommandHandler for QuitCommand {
    fn execute(&self) {
        println!("Good bye !");
        process::exit(0);
    }
}