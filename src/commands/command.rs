/*
Command interface
 */
pub trait CommandHandler {
    fn execute(&self, args: &[String]) -> String;
}