/*
Say hello!
 */
use crate::commands::command::CommandHandler;

pub struct HelloCommand;

impl CommandHandler for HelloCommand {
    fn execute(&self) -> String{
        String::from("Hello !")
    }
}