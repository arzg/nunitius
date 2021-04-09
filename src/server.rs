mod connection_handler;
mod nickname_handler;
mod sender_handler;
mod viewer_handler;

pub use connection_handler::handle_connection;
pub use nickname_handler::nickname_handler;
pub use sender_handler::sender_handler;
pub use viewer_handler::viewer_handler;

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
