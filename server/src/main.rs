use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use shared::{ClientMessage, Lobby, ServerMessage};
use std::{
    collections::HashMap,
    env,
    sync::{Arc, Mutex},
};
use tokio::{
    net,
    sync::broadcast::{channel, Receiver, Sender},
};
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::prelude::*;

#[derive(Clone)]
struct AppState {
    lobbies: Arc<Mutex<HashMap<Lobby, ChannelSend>>>,
}

type ChannelSend = Sender<ServerMessage>;

type ChannelReceive = Receiver<ServerMessage>;

type SocketSend = SplitSink<WebSocket, Message>;

type SocketReceive = SplitStream<WebSocket>;

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
    info!("listening on http://{}", url);

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
    info!("lobby: {}, username: {}", lobby, username);
    let channel_send = state
        .lobbies
        .lock()
        .unwrap()
        .entry(Lobby(lobby))
        .or_insert_with(|| {
            let (channel_send, _) = channel(10);
            channel_send
        })
        .clone();
    ws.on_upgrade(move |socket| websocket(socket, channel_send))
}

async fn websocket(stream: WebSocket, mut channel_send: ChannelSend) {
    info!("websocket opened");

    let (mut socket_send, mut socket_receive) = stream.split();

    let mut channel_receive = channel_send.subscribe();
    let mut handle_channel = tokio::spawn(async move {
        while let Ok(msg) = channel_receive.recv().await {
            info!("sending message: {:?}", msg);
            let bytes = bincode::serialize(&msg).unwrap();
            socket_send.send(Message::Binary(bytes)).await.unwrap();
        }
    });

    let mut handle_socket = tokio::spawn(async move {
        while let Some(msg) = socket_receive.next().await {
            match msg {
                Ok(Message::Binary(bytes)) => match bincode::deserialize(&bytes) {
                    Ok(msg) => {
                        info!("received message: {:?}", msg);
                        receive(msg, &mut channel_send).await;
                    }
                    Err(err) => {
                        info!("message error: {:?}", err);
                    }
                },
                other => {
                    info!("unknown message: {:?}", other);
                }
            }
        }
    });

    tokio::select! {
        _ = &mut handle_channel => {
            tracing::info!("channel closed, aborting websocket");
            handle_socket.abort();
        }
        _ = &mut handle_socket => {
            tracing::info!("websocket closed, aborting channel");
            handle_channel.abort();
        }
    }

    info!("websocket closed");
}

async fn receive(msg: ClientMessage, channel_send: &mut ChannelSend) {
    match msg {
        ClientMessage::HelloFromClient => {
            channel_send.send(ServerMessage::HelloFromServer).unwrap();
        }
    }
}
