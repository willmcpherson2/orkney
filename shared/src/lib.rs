use bevy::ecs::system::Resource;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Resource)]
pub struct Username(pub String);

#[derive(Serialize, Deserialize, Debug, Resource)]
pub struct Lobby(pub String);

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    HelloFromClient,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerMessage {
    HelloFromServer,
}
