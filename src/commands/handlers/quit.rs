/*
Quit application
 */
use crate::commands::command::CommandHandler;
use std::process;

pub struct QuitCommand;

impl CommandHandler for QuitCommand {
    fn execute(&self, _: &[String]) -> String{
        process::exit(0);
    }
}