use axum::Router;
use std::env;
use tokio::net;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let port = env::var("PORT").unwrap_or("3000".to_string());
    let root = env::var("ROOT").unwrap_or("./".to_string());
    let url = format!("localhost:{}", port);
    let router = Router::new().nest_service("/", ServeDir::new(root));
    let listener = net::TcpListener::bind(url).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
