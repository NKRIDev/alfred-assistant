/*
Command interface
 */
use std::collections::HashMap;
use serde_json::{Value};

pub trait CommandHandler : Send + Sync {
    fn execute(&self, args: &HashMap<String, String>) -> String;

    /*
    JSON description of the command, used for
    the AI call tool
     */
    fn description(&self) -> Value;
}