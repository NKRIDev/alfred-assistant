/*
Command interface
 */
pub trait CommandHandler {
    fn execute(&self) -> String;
}