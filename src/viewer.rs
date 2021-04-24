mod protocol;
mod timeline;

pub use protocol::Protocol;
pub use timeline::Timeline;

use crate::{Color, Event as ServerEvent, EventKind as ServerEventKind, Message, User};
use chrono::{DateTime, Local, Utc};
use crossterm::style::{self, style, Styler};

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

pub fn render_event(
    Event {
        event,
        user,
        time_occurred,
    }: &Event,
) -> String {
    let user = render_user(user);

    let local_time_occurred = time_occurred.with_timezone(&Local);
    let local_time_occurred = local_time_occurred.format("%H:%M");

    match event {
        EventKind::Message(Message { body }) => {
            format!("[{}] {}: {}", local_time_occurred, user, body)
        }
        EventKind::Login => format!("[{}] {} logged in!", local_time_occurred, user),
        EventKind::Logout => format!("[{}] {} logged out!", local_time_occurred, user),
    }
}

pub fn render_currently_typing_users<'a>(
    mut users: impl Iterator<Item = &'a User> + ExactSizeIterator,
) -> String {
    let num_users = users.len();

    if num_users == 1 {
        format!("{} is typing...", render_user(users.next().unwrap()))
    } else {
        let users = users.map(render_user).collect::<Vec<_>>().join(" and ");
        format!("{} are typing...", users)
    }
}

fn render_user(user: &User) -> String {
    let base_styled_content = style(&user.nickname).bold();

    if let Some(ref color) = user.color {
        let color = match color {
            Color::Red => style::Color::Red,
            Color::Green => style::Color::Green,
            Color::Yellow => style::Color::Yellow,
            Color::Blue => style::Color::Blue,
            Color::Magenta => style::Color::Magenta,
            Color::Cyan => style::Color::Cyan,
        };

        base_styled_content.with(color).to_string()
    } else {
        base_styled_content.to_string()
    }
}
