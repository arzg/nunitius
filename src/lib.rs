pub mod server;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    Message(Message),
    Login(Login),
    Logout { nickname: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub body: String,
    pub author: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Login {
    pub nickname: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub nickname_taken: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ConnectionKind {
    Sender,
    Viewer,
}
