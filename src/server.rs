mod connection_handler;
mod history_handler;
mod nickname_handler;
mod sender_handler;
mod viewer_handler;

pub use connection_handler::handle_connection;
pub use history_handler::history_handler;
pub use nickname_handler::nickname_handler;
pub use sender_handler::sender_handler;
pub use viewer_handler::viewer_handler;

use crate::Event;
use flume::Sender;

pub enum NicknameEvent {
    Login {
        nickname: String,
        is_taken_tx: Sender<bool>,
    },
    Logout {
        nickname: String,
    },
}

pub struct HistoryRequest {
    history_tx: Sender<Vec<Event>>,
}
