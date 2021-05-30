pub mod sender;
pub mod viewer;

use sender::{LoggingIn, SenderConnection};
use std::io::BufReader;
use std::net::{SocketAddr, TcpListener};
use viewer::{SendingPastEvents, ViewerConnection};

pub fn listen(
    listener: &TcpListener,
    mut sender_handler: impl FnMut(SenderConnection<LoggingIn>, Option<SocketAddr>),
    mut viewer_handler: impl FnMut(ViewerConnection<SendingPastEvents>, Option<SocketAddr>),
    mut error_handler: impl FnMut(anyhow::Error),
) -> ! {
    loop {
        if let Err(e) = accept(&listener, &mut sender_handler, &mut viewer_handler) {
            error_handler(e);
        }
    }
}

fn accept(
    listener: &TcpListener,
    mut sender_handler: impl FnMut(SenderConnection<LoggingIn>, Option<SocketAddr>),
    mut viewer_handler: impl FnMut(ViewerConnection<SendingPastEvents>, Option<SocketAddr>),
) -> anyhow::Result<()> {
    let (mut stream, _) = listener.accept()?;
    let connection_kind = jsonl::read(BufReader::new(&mut stream))?;
    let peer_address = stream.peer_addr().ok();

    match connection_kind {
        model::ConnectionKind::Sender => {
            let sender_connection = SenderConnection::new(stream);
            sender_handler(sender_connection, peer_address);
        }
        model::ConnectionKind::Viewer => {
            let viewer_connection = ViewerConnection::new(stream);
            viewer_handler(viewer_connection, peer_address);
        }
    }

    Ok(())
}
