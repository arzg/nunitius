use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
    Message(Message),
    Login(Login),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message(pub String);

#[derive(Debug, Serialize, Deserialize)]
pub struct Login {
    pub nickname: String,
}
