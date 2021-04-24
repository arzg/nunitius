mod protocol;
mod timeline;
mod ui;

pub use protocol::Protocol;
pub use timeline::Timeline;
pub use ui::{render_currently_typing_users, render_event};

use crate::{Event as ServerEvent, EventKind as ServerEventKind, Message, User};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub event: EventKind,
    pub user: User,
    pub time_occurred: DateTime<Utc>,
}

impl Event {
    fn from_server_event(server_event: ServerEvent) -> Option<Self> {
        Some(Self {
            event: match server_event.event {
                ServerEventKind::Message(msg) => EventKind::Message(msg),
                ServerEventKind::Login => EventKind::Login,
                ServerEventKind::Logout => EventKind::Logout,
                ServerEventKind::Typing(_) => return None,
            },
            user: server_event.user,
            time_occurred: server_event.time_occurred,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventKind {
    Message(Message),
    Login,
    Logout,
}
