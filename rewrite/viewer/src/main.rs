use std::net::SocketAddr;
use std::str::FromStr;
use std::thread;
use viewer_protocol::Protocol;

fn main() -> anyhow::Result<()> {
    let (event_tx, event_rx) = flume::unbounded();

    thread::spawn(|| {
        for event in event_rx {
            dbg!(event);
        }
    });

    let protocol = Protocol::connect(SocketAddr::from_str("127.0.0.1:9292").unwrap(), event_tx)?;
    protocol.receive_events()?;

    Ok(())
}
