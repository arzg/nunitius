use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ConnectionKind {
    Sender,
    Viewer,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub payload: EventPayload,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EventPayload {
    Message(Message),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub nickname: String,
}
