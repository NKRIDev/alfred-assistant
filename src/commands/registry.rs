/*
Register assistant commands
 */
use std::collections::HashMap;
use crate::commands::command::CommandHandler;
use crate::commands::handlers::hello::HelloCommand;
use crate::commands::handlers::open::OpenCommand;
use crate::commands::handlers::quit::QuitCommand;
use crate::commands::handlers::time::TimeCommand;
use crate::commands::handlers::weather::WeatherCommand;
use crate::services::application::ApplicationService;

pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn CommandHandler>>,
}

impl CommandRegistry {

    /*
    Create a new hashmap in constructor
     */
    pub fn new() -> Self {
        Self{
            commands: HashMap::new(),
        }
    }

    /*
    Register a new command with name and command handler interface
     */
    pub fn register(&mut self, name: String, command: Box<dyn CommandHandler>) {
        self.commands.insert(name, command);
    }

    /*
    Execute command by name
     */
    pub fn execute(&self, name: &str, args: &[String]) -> String {
        match self.commands.get(name) {
            Some(command) => command.execute(args),
            None => String::from("Commande inconnue."),
        }
    }
}

/*
Register Alfred commands
 */
pub fn init_commands(app_service : ApplicationService) -> CommandRegistry{
    let mut command_registry = CommandRegistry::new();

    command_registry.register(String::from("hello"), Box::new(HelloCommand));
    command_registry.register(String::from("time"), Box::new(TimeCommand));
    command_registry.register(String::from("open"), Box::new(OpenCommand::new(app_service)));
    command_registry.register(String::from("weather"), Box::new(WeatherCommand));
    command_registry.register(String::from("quit"), Box::new(QuitCommand));

    command_registry
}