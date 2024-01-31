use axum::Router;
use std::env;
use tokio::net;
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let port = env::var("HOST").unwrap_or("0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or("3000".to_string());
    let root = env::var("ROOT").unwrap_or("./".to_string());
    let url = format!("{}:{}", host, port);
    info!("listening on http://{}", url);

    let router = Router::new()
        .nest_service("/", ServeDir::new(root))
        .layer(tower_http::trace::TraceLayer::new_for_http());
    let listener = net::TcpListener::bind(url).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
