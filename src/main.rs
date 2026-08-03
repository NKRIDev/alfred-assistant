pub mod cli;
pub mod commands;
mod services;
mod core;
use dotenvy::dotenv;
use crate::services::application::ApplicationService;

fn main() {
    //Init .env
    dotenv().ok();

    //init application register
    let app_service = ApplicationService::new("apps.toml");

    let system_prompt = String::from("Tu es Alfred un assistant IA");

    //Start alfred loop
    cli::start_alfred(app_service, system_prompt);
}