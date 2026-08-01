/*
Quit application
 */
use crate::commands::command::CommandHandler;
use std::process;

pub struct QuitCommand;

impl CommandHandler for QuitCommand {
    fn execute(&self) -> String{
        process::exit(0);
    }
}