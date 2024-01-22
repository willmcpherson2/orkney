pub type Id = u64;

pub enum ClientMessage {
    RequestId,
}

pub enum ServerMessage {
    NewId(Id),
}
