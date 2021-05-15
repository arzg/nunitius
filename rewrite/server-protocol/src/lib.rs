pub mod sender;
pub mod viewer;

use flume::Sender;
use never::Never;
use sender::{LoggingIn, SenderConnection};
use std::io::BufReader;
use std::net::{SocketAddr, TcpListener};
use viewer::{SendingPastEvents, ViewerConnection};

pub fn listen(
    address: SocketAddr,
    sender_tx: Sender<SenderConnection<LoggingIn>>,
    viewer_tx: Sender<ViewerConnection<SendingPastEvents>>,
) -> anyhow::Result<Never> {
    let listener = TcpListener::bind(address)?;

    loop {
        let (mut stream, _) = listener.accept()?;
        let connection_kind = jsonl::read(BufReader::new(&mut stream))?;

        match connection_kind {
            model::ConnectionKind::Sender => sender_tx.send(SenderConnection::new(stream)).unwrap(),
            model::ConnectionKind::Viewer => viewer_tx.send(ViewerConnection::new(stream)).unwrap(),
        }
    }
}
