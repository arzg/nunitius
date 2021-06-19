use jsonl::Connection;
use std::io::BufReader;
use std::net::{SocketAddr, TcpListener, TcpStream};

type TcpConnection = Connection<BufReader<TcpStream>, TcpStream>;

pub fn listen(
    listener: &TcpListener,
    mut sender_handler: impl FnMut(TcpConnection, Option<SocketAddr>),
    mut viewer_handler: impl FnMut(TcpConnection, Option<SocketAddr>),
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
    mut sender_handler: impl FnMut(TcpConnection, Option<SocketAddr>),
    mut viewer_handler: impl FnMut(TcpConnection, Option<SocketAddr>),
) -> anyhow::Result<()> {
    let (stream, _) = listener.accept()?;
    let peer_address = stream.peer_addr().ok();
    let mut connection = Connection::new_from_tcp_stream(stream)?;

    let connection_kind = connection.read()?;
    match connection_kind {
        model::ConnectionKind::Sender => sender_handler(connection, peer_address),
        model::ConnectionKind::Viewer => viewer_handler(connection, peer_address),
    }

    Ok(())
}
