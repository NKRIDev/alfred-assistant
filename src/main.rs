pub mod cli;
pub mod commands;
mod services;
mod core;

use dotenvy::dotenv;
use crate::commands::registry::init_commands;
use crate::core::alfred::Alfred;
use crate::services::application::ApplicationService;

fn main() {
    //Init .env
    dotenv().ok();

    //init application register
    let app_service = ApplicationService::new("apps.toml");

    //init register command
    let registry = init_commands(app_service);

    //init alfred core
    let alfred = Alfred::new("qwen3:4b-instruct".to_string(), "skills/alfred.md".to_string(),
                             registry);

    //Start alfred loop
    cli::start_alfred(&alfred);
}