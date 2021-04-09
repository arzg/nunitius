use crate::ConnectionKind;
use flume::Sender;
use log::info;
use std::io;
use std::net::TcpStream;

pub fn handle_connection(
    stream: TcpStream,
    sender_tx: Sender<TcpStream>,
    viewer_tx: Sender<TcpStream>,
) -> anyhow::Result<()> {
    let mut stream = io::BufReader::new(stream);
    let connection_kind = jsonl::read(&mut stream)?;
    let stream = stream.into_inner();

    info!("connection kind: {:?}", connection_kind);

    match connection_kind {
        ConnectionKind::Sender => sender_tx.send(stream).unwrap(),
        ConnectionKind::Viewer => viewer_tx.send(stream).unwrap(),
    }

    Ok(())
}
