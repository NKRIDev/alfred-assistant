/*
Register assistant commands
 */
use std::collections::HashMap;
use std::env;
use serde_json::Value;
use crate::commands::command::CommandHandler;
use crate::commands::handlers::calendar::create_event::CreateEventCommand;
use crate::commands::handlers::calendar::delete_event::DeleteEventCommand;
use crate::commands::handlers::calendar::list_events::ListEventsCommand;
use crate::commands::handlers::calendar::update_event::UpdateEventCommand;
use crate::commands::handlers::gmail::draft_reply::DraftReplyCommand;
use crate::commands::handlers::gmail::list_unread_emails::ListUnreadEmailsCommand;
use crate::commands::handlers::gmail::manage_email::ManageEmailCommand;
use crate::commands::handlers::gmail::search_emails::SearchEmailsCommand;
use crate::commands::handlers::open::OpenCommand;
use crate::commands::handlers::quit::QuitCommand;
use crate::commands::handlers::search::SearchCommand;
use crate::commands::handlers::time::TimeCommand;
use crate::commands::handlers::weather::WeatherCommand;
use crate::services::application::ApplicationService;
use crate::services::calendar::CalendarService;
use crate::services::gmail::GmailService;
use crate::services::google_auth::GoogleAuthService;

pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn CommandHandler + Sync + Send>>,
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
    pub fn execute(&self, name: &str, args: &HashMap<String, String>) -> String {
        match self.commands.get(name) {
            Some(command) => command.execute(args),
            None => String::from("Commande inconnue."),
        }
    }

    /*
    Create the list of tools (alfred's CLI command)
    in JSON
     */
    pub fn build_tools(&self) -> Value {
        let tools: Vec<Value> = self.commands.values()
            .map(|cmd| cmd.description()).collect();
        Value::Array(tools)
    }
}

/*
Register Alfred commands
 */
pub fn init_commands(app_service : ApplicationService) -> CommandRegistry{
    let mut command_registry = CommandRegistry::new();

    command_registry.register(String::from("time"), Box::new(TimeCommand));
    command_registry.register(String::from("open"), Box::new(OpenCommand::new(app_service)));
    command_registry.register(String::from("weather"), Box::new(WeatherCommand));
    command_registry.register(String::from("search"), Box::new(SearchCommand));
    command_registry.register(String::from("quit"), Box::new(QuitCommand));

    /*
    Init googles features
     */
    let google_auth = GoogleAuthService::new(
        env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID manquant"),
        env::var("GOOGLE_CLIENT_SECRET").expect("GOOGLE_CLIENT_SECRET manquant"),
        env::var("GOOGLE_REFRESH_TOKEN").expect("GOOGLE_REFRESH_TOKEN manquant"),
    );

    let gmail = GmailService::new(google_auth.clone());
    command_registry.register(String::from("list_unread_emails"), Box::new(ListUnreadEmailsCommand::new(gmail.clone())));
    command_registry.register(String::from("search_emails"), Box::new(SearchEmailsCommand::new(gmail.clone())));
    command_registry.register(String::from("manage_email"), Box::new(ManageEmailCommand::new(gmail.clone())));
    command_registry.register(String::from("draft_reply"), Box::new(DraftReplyCommand::new(gmail)));

    let calendar = CalendarService::new(google_auth);
    command_registry.register(String::from("list_events"), Box::new(ListEventsCommand::new(calendar.clone())));
    command_registry.register(String::from("create_event"), Box::new(CreateEventCommand::new(calendar.clone())));
    command_registry.register(String::from("update_event"), Box::new(UpdateEventCommand::new(calendar.clone())));
    command_registry.register(String::from("delete_event"), Box::new(DeleteEventCommand::new(calendar)));

    command_registry
}