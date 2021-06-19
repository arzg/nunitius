use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConnectionKind {
    Sender,
    Viewer,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub data: EventData,
    pub user: User,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventData {
    Message(Message),
    Login,
    Logout,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SenderRequest {
    Login(User),
    NewMessage(Message),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LoginResponse {
    Succeeded,
    Taken,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub nickname: String,
}
