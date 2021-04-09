use log::error;
use std::net::TcpListener;
use std::thread;

fn main() -> anyhow::Result<()> {
    fern::Dispatch::new()
        .level(log::LevelFilter::Trace)
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                record.level(),
                record.target(),
                message,
            ))
        })
        .chain(std::io::stdout())
        .apply()?;

    let listener = TcpListener::bind("127.0.0.1:9999")?;

    let (sender_tx, sender_rx) = flume::bounded(100);
    let (viewer_tx, viewer_rx) = flume::bounded(100);
    let (event_tx, event_rx) = flume::bounded(100);
    let (nickname_event_tx, nickname_event_rx) = flume::bounded(100);

    thread::spawn(|| nunitius::server::sender_handler(sender_rx, nickname_event_tx, event_tx));
    thread::spawn(|| nunitius::server::viewer_handler(event_rx, viewer_rx));
    thread::spawn(|| nunitius::server::nickname_handler(nickname_event_rx));

    for stream in listener.incoming() {
        let stream = stream?;
        let sender_tx = sender_tx.clone();
        let viewer_tx = viewer_tx.clone();

        if let Err(e) = nunitius::server::handle_connection(stream, sender_tx, viewer_tx) {
            error!("{:#}", e);
        }
    }

    Ok(())
}
