use axum::Router;
use axum::routing::get;

async fn hello_world() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    /*
    Init routes API
     */
    let app = Router::new().route("/hello", get(hello_world));

    /*
    Start web server
     */
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
