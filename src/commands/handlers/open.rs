/*
Launch an application
 */
use crate::commands::command::CommandHandler;
use crate::services::application::ApplicationService;

pub struct OpenCommand {
    app_service: ApplicationService,
}

impl OpenCommand {
    pub fn new(app_service: ApplicationService) -> OpenCommand {
        OpenCommand { app_service }
    }
}

impl CommandHandler for OpenCommand {
    fn execute(&self, args: &[String]) -> String{
        if args.len() < 1 {
            return String::from("Missing argument; specify the name of the app to launch.");
        }

        self.app_service.open(&args[0])
    }
}