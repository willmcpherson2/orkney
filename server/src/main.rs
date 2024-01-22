use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::StreamExt;
use shared::Id;
use std::{
    env,
    sync::{Arc, Mutex},
};
use tokio::{net, sync::broadcast};
use tower_http::services::ServeDir;
use tracing_subscriber::prelude::*;

struct AppState {
    id_counter: Mutex<Id>,
    tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let port = env::var("PORT").unwrap_or("3000".to_string());
    let root = env::var("ROOT").unwrap_or("./".to_string());
    let url = format!("localhost:{}", port);

    tracing::debug!("listening on http://{}", url);

    let id_counter = Mutex::new(0);
    let (tx, _rx) = broadcast::channel(100);
    let app_state = Arc::new(AppState { id_counter, tx });

    let router = Router::new()
        .nest_service("/", ServeDir::new(root))
        .route("/ws", get(websocket_handler))
        .with_state(app_state)
        .layer(tower_http::trace::TraceLayer::new_for_http());
    let listener = net::TcpListener::bind(url).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

async fn websocket(mut stream: WebSocket, state: Arc<AppState>) {
    while let Some(Ok(message)) = stream.next().await {
        if let Message::Text(ping) = message {
            println!("received message: {:?}", ping);
            stream
                .send(Message::Text(String::from("pong")))
                .await
                .unwrap();
            println!("sent a pong");
        }
    }
}
