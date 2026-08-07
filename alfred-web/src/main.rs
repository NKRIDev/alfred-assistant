pub mod controllers;
pub mod dtos;
pub mod services;

use axum::{Router, routing::post};
use alfred_core::core::alfred::Alfred;
use alfred_core::commands::registry::init_commands;
use alfred_core::services::application::ApplicationService;
use crate::controllers::ask_controller::ask_handle;
use crate::services::ask_service::AskService;
use std::sync::Arc;
use tokio::sync::Mutex;
use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    //Init .env
    dotenv().ok();

    //init application register
    let app_service = ApplicationService::new("apps.toml");

    //init register command
    let registry = init_commands(app_service);

    //init alfred core
    let alfred = Alfred::new("qwen3:4b-instruct".to_string(), "skills/alfred.md".to_string(),
                             "datas/events.db".to_string(), registry);

    let shared_alfred = Arc::new(Mutex::new(alfred));
    let ask_service = AskService::new(shared_alfred);

    /*
    Init routes API
    */
    let app = Router::new()
        .route("/ask", post(ask_handle))
        .with_state(ask_service);

    /*
    Start web server
     */
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
