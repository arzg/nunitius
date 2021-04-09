use std::net::TcpListener;
use std::thread;
use tracing::{error, span, Level};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    let listener = TcpListener::bind("127.0.0.1:9999")?;

    let (viewer_tx, viewer_rx) = flume::bounded(100);
    let (events_tx, events_rx) = flume::bounded(100);
    let (nickname_event_tx, nickname_event_rx) = flume::bounded(100);

    thread::spawn(|| nunitius::server::viewer_handler(events_rx, viewer_rx));
    thread::spawn(|| nunitius::server::nickname_handler(nickname_event_rx));

    for stream in listener.incoming() {
        let stream = stream?;
        let viewer_tx = viewer_tx.clone();
        let events_tx = events_tx.clone();
        let nickname_event_tx = nickname_event_tx.clone();

        thread::spawn(|| {
            let span = span!(
                Level::INFO,
                "handling_connection",
                addr = debug(stream.peer_addr()),
            );
            let _guard = span.enter();

            if let Err(e) =
                nunitius::server::handle_connection(stream, viewer_tx, events_tx, nickname_event_tx)
            {
                error!("{:#}", e);
            }
        });
    }

    Ok(())
}
