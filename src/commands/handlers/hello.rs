/*
Say hello!
 */
use crate::commands::command::CommandHandler;

pub struct HelloCommand;

impl CommandHandler for HelloCommand {
    fn execute(&self, _: &[String]) -> String{
        String::from("Hello !")
    }
}