pub mod controllers;
pub mod dtos;
pub mod services;

use std::env;
use axum::extract::FromRef;
use axum::{Router, routing::post};
use alfred_core::core::alfred::Alfred;
use alfred_core::commands::registry::init_commands;
use alfred_core::services::application::ApplicationService;
use crate::controllers::ask_controller::{ask_handle, ask_micro_handle};
use crate::services::ask_service::AskService;
use std::sync::Arc;
use axum::routing::get;
use tokio::sync::Mutex;
use dotenvy::dotenv;
use tower_http::cors::{Any, CorsLayer};
use alfred_core::services::gmail::GmailService;
use alfred_core::services::google_auth::GoogleAuthService;
use alfred_core::services::notification_store::NotificationStore;
use alfred_core::services::stt::SttService;
use alfred_core::services::tts::TtsService;
use alfred_core::watchers::handlers::gmail::GmailWatcher;
use alfred_core::watchers::scheduler::WatcherScheduler;
use crate::controllers::notifications::notifications_handle;

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

    //init tts service
    let tts = TtsService::new();

    //STT service
    let stt = SttService::new("stt-models/ggml-small.bin").expect("Model not found");

    let shared_alfred = Arc::new(Mutex::new(alfred));
    let ask_service = AskService::new(shared_alfred.clone(), tts, stt);

    /*
    CORS
    */
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    /*
    Start notifications sheduler
     */
    let google_auth = GoogleAuthService::new(
        env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID manquant"),
        env::var("GOOGLE_CLIENT_SECRET").expect("GOOGLE_CLIENT_SECRET manquant"),
        env::var("GOOGLE_REFRESH_TOKEN").expect("GOOGLE_REFRESH_TOKEN manquant"),
    );
    let gmail_for_watcher = GmailService::new(google_auth);
    let notifications = NotificationStore::new();
    WatcherScheduler::start(
        shared_alfred.clone(),
        vec![Box::new(GmailWatcher::new(gmail_for_watcher))],
        notifications.clone(),
        30,
    );

    /*
    Init routes API
    */
    let state = AppState {ask_service, notifications};
    let app = Router::new()
        .route("/ask", post(ask_handle))
        .route("/ask-audio", post(ask_micro_handle))
        .route("/notifications", get(notifications_handle))
        .layer(cors)
        .with_state(state);

    /*
    Start web server
     */
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[derive(Clone)]
pub struct AppState {
    pub ask_service: AskService,
    pub notifications: NotificationStore,
}

impl FromRef<AppState> for AskService {
    fn from_ref(state: &AppState) -> Self {
        state.ask_service.clone()
    }
}

impl FromRef<AppState> for NotificationStore {
    fn from_ref(state: &AppState) -> Self {
        state.notifications.clone()
    }
}