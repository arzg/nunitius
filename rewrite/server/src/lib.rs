use server_protocol::sender::{LoggingIn, SenderConnection};
use server_protocol::viewer::{SendingPastEvents, ViewerConnection};

pub fn handle_sender(_sender_connection: SenderConnection<LoggingIn>) -> anyhow::Result<()> {
    Ok(())
}

pub fn handle_viewer(
    _viewer_connection: ViewerConnection<SendingPastEvents>,
) -> anyhow::Result<()> {
    Ok(())
}
