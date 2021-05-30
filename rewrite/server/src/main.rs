use flume::{Receiver, Sender};
use std::net::{SocketAddr, TcpListener};
use std::thread;

fn main() -> anyhow::Result<()> {
    let (error_tx, error_rx) = flume::unbounded();
    thread::spawn(|| error_printer(error_rx));

    server_protocol::listen(
        &TcpListener::bind("127.0.0.1:9292")?,
        |sender_connection, peer_address| {
            let error_tx = error_tx.clone();

            thread::spawn(move || {
                let result = nunitius_server::handle_sender(sender_connection);
                handle_error(result, HandlerKind::Sender, peer_address, error_tx);
            });
        },
        |viewer_connection, peer_address| {
            let error_tx = error_tx.clone();

            thread::spawn(move || {
                let result = nunitius_server::handle_viewer(viewer_connection);
                handle_error(result, HandlerKind::Viewer, peer_address, error_tx);
            });
        },
        print_error,
    );
}

enum HandlerKind {
    Sender,
    Viewer,
}

fn handle_error(
    result: anyhow::Result<()>,
    handler_kind: HandlerKind,
    peer_address: Option<SocketAddr>,
    error_tx: Sender<anyhow::Error>,
) {
    if let Err(error) = result {
        let error = error.context(compute_handler_error_context(handler_kind, peer_address));
        error_tx.send(error).unwrap();
    }
}

fn compute_handler_error_context(
    handler_kind: HandlerKind,
    peer_address: Option<SocketAddr>,
) -> String {
    let mut output = "failed to handle ".to_string();

    match handler_kind {
        HandlerKind::Sender => output.push_str("sender"),
        HandlerKind::Viewer => output.push_str("viewer"),
    }

    if let Some(a) = peer_address {
        output.push_str(&format!(" at address {}", a));
    }

    output
}

fn error_printer(error_rx: Receiver<anyhow::Error>) {
    for e in error_rx {
        print_error(e);
    }
}

fn print_error(e: anyhow::Error) {
    eprintln!("Error: {:?}", e);
}
