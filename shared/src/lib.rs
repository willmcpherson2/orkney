use serde::{Serialize, Deserialize};

pub type Id = u64;

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    RequestId,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    NewId(Id),
}
