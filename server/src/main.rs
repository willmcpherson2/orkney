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
use shared::{ClientMessage, ServerMessage};
use std::{
    env,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use tokio::net;
use tower_http::services::ServeDir;
use tracing_subscriber::prelude::*;

struct AppState {
    id_counter: AtomicU64,
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

    let id_counter = AtomicU64::new(0);
    let app_state = Arc::new(AppState { id_counter });

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
    tracing::info!("starting websocket");
    while let Some(msg) = stream.next().await {
        match msg {
            Ok(Message::Binary(bytes)) => match bincode::deserialize(&bytes) {
                Ok(msg) => {
                    tracing::info!("received message: {:?}", msg);
                    receive(msg, &state, &mut stream).await;
                }
                Err(err) => {
                    tracing::info!("message error: {:?}", err);
                }
            },
            other => {
                tracing::info!("unknown message: {:?}", other);
            }
        }
    }
}

async fn receive(msg: ClientMessage, state: &Arc<AppState>, stream: &mut WebSocket) {
    match msg {
        ClientMessage::RequestId => {
            let id = state.id_counter.fetch_add(1, Ordering::SeqCst);
            let msg = ServerMessage::NewId(id);
            let bytes = bincode::serialize(&msg).unwrap();
            stream.send(Message::Binary(bytes)).await.unwrap();
            tracing::info!("sent message: {:?}", msg);
        }
    }
}
