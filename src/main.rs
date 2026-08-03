pub mod cli;
pub mod commands;
mod services;

use dotenvy::dotenv;
use crate::services::application::ApplicationService;

fn main() {
    //Init .env
    dotenv().ok();

    //init application register
    let app_service = ApplicationService::new("apps.toml");

    //Start alfred loop
    cli::start_alfred(app_service);
}