use axum::{Json, Router};
use axum::routing::{get, post};
use serde::Deserialize;

async fn hello_world() -> &'static str {
    "Hello, World!"
}

/*
Body to ask content
 */
#[derive(Deserialize)]
struct AlfredAsk {
    text: String,
}

/*
POST on /ask to recover text input by user
 */
async fn ask_handle(Json(payload): Json<AlfredAsk>) {
    println!("{}", payload.text);
}

#[tokio::main]
async fn main() {
    /*
    Init routes API
     */
    let app = Router::new().route("/hello", get(hello_world))
        .route("/ask", post(ask_handle));

    /*
    Start web server
     */
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
