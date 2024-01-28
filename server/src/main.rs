use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use shared::{ClientMessage, Lobby, ServerMessage};
use std::{
    collections::HashMap,
    env,
    sync::{Arc, Mutex},
};
use tokio::{
    net,
    sync::broadcast::{channel, Sender},
};
use tower_http::services::ServeDir;
use tracing_subscriber::prelude::*;

#[derive(Clone)]
struct AppState {
    lobbies: Arc<Mutex<HashMap<Lobby, Clients>>>,
}

type Clients = Sender<ServerMessage>;

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

    let state = AppState {
        lobbies: Arc::new(Mutex::new(HashMap::new())),
    };

    let router = Router::new()
        .nest_service("/", ServeDir::new(root))
        .route("/join/:lobby/:username", get(websocket_handler))
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());
    let listener = net::TcpListener::bind(url).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

async fn websocket_handler(
    Path((lobby, username)): Path<(String, String)>,
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    tracing::info!("lobby: {}, username: {}", lobby, username);
    let clients = state
        .lobbies
        .lock()
        .unwrap()
        .entry(Lobby(lobby))
        .or_insert_with(|| {
            let (tx, _) = channel(10);
            tx
        })
        .clone();
    ws.on_upgrade(move |socket| websocket(socket, clients))
}

async fn websocket(stream: WebSocket, mut clients: Clients) {
    tracing::info!("websocket opened");

    let (mut client_outbox, mut client_inbox) = stream.split();

    let mut clients_inbox = clients.subscribe();
    let mut handle_clients_inbox = tokio::spawn(async move {
        while let Ok(msg) = clients_inbox.recv().await {
            tracing::info!("sending message: {:?}", msg);
            let bytes = bincode::serialize(&msg).unwrap();
            client_outbox.send(Message::Binary(bytes)).await.unwrap();
        }
    });

    let mut handle_client_inbox = tokio::spawn(async move {
        while let Some(msg) = client_inbox.next().await {
            match msg {
                Ok(Message::Binary(bytes)) => match bincode::deserialize(&bytes) {
                    Ok(msg) => {
                        tracing::info!("received message: {:?}", msg);
                        receive(msg, &mut clients).await;
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
    });

    tokio::select! {
        _ = &mut handle_clients_inbox => {
            tracing::info!("clients inbox closed, closing client inbox");
            handle_client_inbox.abort();
        }
        _ = &mut handle_client_inbox => {
            tracing::info!("client inbox closed, closing clients inbox");
            handle_clients_inbox.abort();
        }
    }

    tracing::info!("websocket closed");
}

async fn receive(msg: ClientMessage, clients_outbox: &mut Sender<ServerMessage>) {
    match msg {
        ClientMessage::HelloFromClient => {
            clients_outbox.send(ServerMessage::HelloFromServer).unwrap();
        }
    }
}
