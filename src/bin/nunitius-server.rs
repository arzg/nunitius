use flume::{Receiver, Sender};
use log::error;
use std::net::TcpListener;
use std::thread;

fn main() -> anyhow::Result<()> {
    fern::Dispatch::new()
        .level(log::LevelFilter::Trace)
        .format(|out, message, record| {
            out.finish(format_args!(
                "{:<7} {:<40} {}",
                format!("[{}]", record.level()),
                format!("[{}]", record.target()),
                message,
            ))
        })
        .chain(std::io::stdout())
        .apply()?;

    let listener = TcpListener::bind("127.0.0.1:9999")?;

    let (sender_tx, sender_rx) = flume::bounded(100);
    let (viewer_tx, viewer_rx) = flume::bounded(100);
    let (nickname_event_tx, nickname_event_rx) = flume::bounded(100);
    let (history_request_tx, history_request_rx) = flume::bounded(100);

    let (event_tx, event_rx) = flume::bounded(100);
    let (viewer_handler_event_tx, viewer_handler_event_rx) = flume::bounded(100);
    let (history_handler_event_tx, history_handler_event_rx) = flume::bounded(100);

    thread::spawn(|| nunitius::server::sender_handler(sender_rx, nickname_event_tx, event_tx));
    thread::spawn(|| {
        nunitius::server::viewer_handler(viewer_rx, viewer_handler_event_rx, history_request_tx)
    });
    thread::spawn(|| nunitius::server::nickname_handler(nickname_event_rx));
    thread::spawn(|| {
        nunitius::server::history_handler(history_handler_event_rx, history_request_rx)
    });

    thread::spawn(|| {
        fanout(
            event_rx,
            &[viewer_handler_event_tx, history_handler_event_tx],
        )
    });

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

fn fanout<T: Clone>(rx: Receiver<T>, txs: &[Sender<T>]) {
    for t in rx {
        for tx in txs {
            tx.send(t.clone()).unwrap();
        }
    }
}
