use bevy::ecs::system::Resource;
use serde::{Serialize, Deserialize};

#[derive(Resource, Serialize, Deserialize, Debug)]
pub struct ClientId(pub u64);

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    RequestClientId,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    NewClientId(ClientId),
}
