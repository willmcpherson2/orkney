// Signaling server code based on matchbox_server by Johan Helsing
// https://github.com/johanhelsing/matchbox/tree/31d3e30e8b329cdadcb6a36e5a8c541f556ad7aa/matchbox_server

mod state;
mod topology;

use axum::Router;
use matchbox_signaling::SignalingServerBuilder;
use std::{env, net::SocketAddr};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::prelude::*;

use crate::{
    state::{RequestedRoom, RoomId, ServerState},
    topology::MatchmakingDemoTopology,
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let host = env::var("HOST").unwrap_or("0.0.0.0".to_string());
    let static_port = env::var("STATIC_SERVER_PORT").unwrap_or("3000".to_string());
    let signaling_port = env::var("SIGNALING_SERVER_PORT").unwrap_or("3001".to_string());
    let static_url: SocketAddr = format!("{}:{}", host, static_port).parse().unwrap();
    let signaling_url: SocketAddr = format!("{}:{}", host, signaling_port).parse().unwrap();
    let root = env::var("ROOT").unwrap_or("./".to_string());

    let app = Router::new()
        .nest_service("/", ServeDir::new(root))
        .layer(TraceLayer::new_for_http())
        .into_make_service();
    let static_server = axum::Server::bind(&static_url).serve(app);

    let mut state = ServerState::default();
    let signaling_server =
        SignalingServerBuilder::new(signaling_url, MatchmakingDemoTopology, state.clone())
            .on_connection_request({
                let mut state = state.clone();
                move |connection| {
                    let room_id = RoomId(connection.path.clone().unwrap_or_default());
                    let room = RequestedRoom { id: room_id };
                    state.add_waiting_client(connection.origin, room);
                    Ok(true) // allow all clients
                }
            })
            .on_id_assignment({
                move |(origin, peer_id)| {
                    info!("Client connected {origin:?}: {peer_id:?}");
                    state.assign_id_to_waiting_client(origin, peer_id);
                }
            })
            .cors()
            .trace()
            .build()
            .serve();

    tokio::select! {
        _ = static_server => {},
        _ = signaling_server => {},
    }
}
