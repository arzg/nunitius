use flume::{Receiver, Sender};
use jsonl::{Connection, ReadError};
use parking_lot::Mutex;
use server_core::Server;
use std::io::BufReader;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

fn main() -> anyhow::Result<()> {
    let (error_tx, error_rx) = flume::unbounded();
    thread::spawn(|| print_errors(error_rx));

    let viewers = Arc::new(Mutex::new(Vec::new()));

    let (event_tx, event_rx) = flume::unbounded();
    let server = Arc::new(Mutex::new(Server::new(event_tx)));
    thread::spawn({
        let error_tx = error_tx.clone();
        let viewers = Arc::clone(&viewers);
        move || forward_events(viewers, event_rx, error_tx)
    });

    server_protocol::listen(
        &TcpListener::bind("127.0.0.1:9292")?,
        |connection, peer_address| {
            let server = Arc::clone(&server);
            let error_tx = error_tx.clone();

            thread::spawn(move || {
                let result = handle_sender(server, connection);
                handle_error(result, HandlerKind::Sender, peer_address, &error_tx);
            });
        },
        |mut connection, peer_address| {
            let server = Arc::clone(&server);
            let result = handle_viewer(server, &mut connection);
            handle_error(result, HandlerKind::Viewer, peer_address, &error_tx);
            viewers.lock().push(connection);
        },
        print_error,
    );
}

type TcpConnection = Connection<BufReader<TcpStream>, TcpStream>;

fn handle_sender(server: Arc<Mutex<Server>>, mut connection: TcpConnection) -> anyhow::Result<()> {
    let mut id = None;

    loop {
        let request: model::SenderRequest = match connection.read() {
            Ok(request) => request,
            Err(ReadError::Eof) => {
                if let Some(id) = id {
                    server.lock().logout(id);
                }
                break;
            }
            Err(e) => return Err(e.into()),
        };

        let request = match request {
            model::SenderRequest::Login(user) => {
                match server.lock().login(user) {
                    server_core::LoginResponse::Succeeded(i) => {
                        id = Some(i);
                        connection.write(&model::LoginResponse::Succeeded)?;
                    }
                    server_core::LoginResponse::Taken => {
                        connection.write(&model::LoginResponse::Taken)?;
                    }
                }
                continue;
            }
            request => request,
        };

        let id = if let Some(id) = id {
            id
        } else {
            connection.write(&"Tried to send request before logging in")?;
            continue;
        };

        match request {
            // login requests have already been handled
            // at this point
            model::SenderRequest::Login(_) => unreachable!(),

            model::SenderRequest::NewMessage(message) => {
                server.lock().send_message(id, message);
            }
        }
    }

    Ok(())
}

fn handle_viewer(server: Arc<Mutex<Server>>, connection: &mut TcpConnection) -> anyhow::Result<()> {
    let server = server.lock();
    let past_events = server.past_events();
    connection.write(&past_events)?;

    Ok(())
}

fn forward_events(
    viewers: Arc<Mutex<Vec<TcpConnection>>>,
    event_rx: Receiver<model::Event>,
    error_tx: Sender<anyhow::Error>,
) {
    for event in event_rx {
        for viewer in &mut *viewers.lock() {
            if let Err(e) = viewer.write(&event) {
                error_tx.send(anyhow::Error::new(e)).unwrap();
            }
        }
    }
}

enum HandlerKind {
    Sender,
    Viewer,
}

fn handle_error(
    result: anyhow::Result<()>,
    handler_kind: HandlerKind,
    peer_address: Option<SocketAddr>,
    error_tx: &Sender<anyhow::Error>,
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

fn print_errors(error_rx: Receiver<anyhow::Error>) {
    for e in error_rx {
        print_error(e);
    }
}

fn print_error(e: anyhow::Error) {
    eprintln!("Error: {:?}", e);
}
