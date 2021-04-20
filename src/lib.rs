pub mod sender;
pub mod server;
pub mod viewer;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event: EventKind,
    pub user: User,
    pub time_occurred: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventKind {
    Message(Message),
    Login,
    Logout,
    Typing(TypingEvent),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SenderEvent {
    Message(Message),
    Typing(TypingEvent),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TypingEvent {
    Start,
    Stop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Login {
    pub user: User,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct User {
    pub nickname: String,
    pub color: Option<Color>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Color {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
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
