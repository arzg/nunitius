use flume::{Receiver, Selector, Sender};
use nunitius::{ConnectionKind, Event, Login, LoginResponse};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::net::{TcpListener, TcpStream};
use std::{io, thread};
use tracing::{error, info, span, Level};

type TcpConnection = jsonl::Connection<io::BufReader<TcpStream>, TcpStream>;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    let listener = TcpListener::bind("127.0.0.1:9999")?;

    let (viewer_tx, viewer_rx) = flume::bounded(100);
    let (events_tx, events_rx) = flume::bounded(100);
    let (nickname_event_tx, nickname_event_rx) = flume::bounded(100);

    thread::spawn(|| viewer_handler(events_rx, viewer_rx));
    thread::spawn(|| nickname_handler(nickname_event_rx));

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

            if let Err(e) = handle_connection(stream, viewer_tx, events_tx, nickname_event_tx) {
                error!("{:#}", e);
            }
        });
    }

    Ok(())
}

fn handle_connection(
    stream: TcpStream,
    viewer_tx: Sender<TcpStream>,
    events_tx: Sender<Event>,
    nickname_event_tx: Sender<NicknameEvent>,
) -> anyhow::Result<()> {
    let mut stream = io::BufReader::new(stream);
    let connection_kind = jsonl::read(&mut stream)?;
    let stream = stream.into_inner();

    info!(?connection_kind);

    match connection_kind {
        ConnectionKind::Sender => handle_sender_connection(stream, nickname_event_tx, events_tx)?,
        ConnectionKind::Viewer => viewer_tx.send(stream).unwrap(),
    }

    Ok(())
}

fn handle_sender_connection(
    stream: TcpStream,
    nickname_event_tx: Sender<NicknameEvent>,
    events_tx: Sender<Event>,
) -> Result<(), anyhow::Error> {
    let mut connection = jsonl::Connection::new_from_tcp_stream(stream)?;
    let nickname = log_sender_in(&mut connection, &nickname_event_tx, &events_tx)?;

    loop {
        match connection.read() {
            Ok(message) => {
                info!("received message");
                events_tx.send(Event::Message(message)).unwrap();
            }

            Err(jsonl::ReadError::Eof) => {
                info!("logged out");

                nickname_event_tx
                    .send(NicknameEvent::Logout { nickname })
                    .unwrap();

                break;
            }

            Err(e) => return Err(e.into()),
        }
    }

    Ok(())
}

fn log_sender_in(
    connection: &mut TcpConnection,
    nickname_event_tx: &Sender<NicknameEvent>,
    events_tx: &Sender<Event>,
) -> anyhow::Result<String> {
    loop {
        let login: Login = connection.read()?;
        info!(?login, "read login from sender");

        let is_nickname_taken =
            check_if_nickname_is_taken(login.nickname.clone(), nickname_event_tx)?;

        connection.write(&LoginResponse {
            nickname_taken: is_nickname_taken,
        })?;

        if is_nickname_taken {
            info!("nickname was taken, retrying");
        } else {
            info!("logged in with unique nickname");
            events_tx.send(Event::Login(login.clone())).unwrap();
            return Ok(login.nickname);
        }
    }
}

fn check_if_nickname_is_taken(
    nickname: String,
    nickname_event_tx: &Sender<NicknameEvent>,
) -> Result<bool, anyhow::Error> {
    let (is_nickname_taken_tx, is_nickname_taken_rx) = flume::bounded(0);

    nickname_event_tx.send(NicknameEvent::Login {
        nickname,
        is_taken_tx: is_nickname_taken_tx,
    })?;

    Ok(is_nickname_taken_rx.recv().unwrap())
}

fn viewer_handler(events_rx: Receiver<Event>, viewer_rx: Receiver<TcpStream>) {
    let span = span!(Level::INFO, "handling_viewers");
    let _guard = span.enter();

    let viewers = RefCell::new(HashMap::new());
    let mut current_viewer_idx = 0;

    loop {
        Selector::new()
            .recv(&viewer_rx, |viewer| {
                info!("received new viewer");
                handle_new_viewer(&viewers, &mut current_viewer_idx, viewer.unwrap());
            })
            .recv(&events_rx, |event| {
                info!("received event");
                handle_new_event(&viewers, event.unwrap());
            })
            .wait();
    }
}

fn handle_new_viewer(
    viewers: &RefCell<HashMap<i32, TcpStream>>,
    current_viewer_idx: &mut i32,
    viewer: TcpStream,
) {
    viewers.borrow_mut().insert(*current_viewer_idx, viewer);
    *current_viewer_idx += 1;
}

fn handle_new_event(viewers: &RefCell<HashMap<i32, TcpStream>>, event: Event) {
    let mut closed_viewers = Vec::new();
    send_events_to_viewers(viewers, event, &mut closed_viewers);
    remove_closed_viewers(viewers, closed_viewers.into_iter());
}

fn send_events_to_viewers(
    viewers: &RefCell<HashMap<i32, TcpStream>>,
    event: Event,
    closed_viewers: &mut Vec<i32>,
) {
    let mut viewers = viewers.borrow_mut();

    for (idx, viewer) in viewers.iter_mut() {
        match jsonl::write(viewer, &event) {
            Ok(()) => info!("forwarded event to viewer"),

            Err(jsonl::WriteError::Io(io_error))
                if io_error.kind() == io::ErrorKind::BrokenPipe =>
            {
                info!("found closed viewer");
                closed_viewers.push(*idx);
            }

            Err(e) => error!("{:#}", anyhow::Error::new(e)),
        }
    }
}

fn remove_closed_viewers(
    viewers: &RefCell<HashMap<i32, TcpStream>>,
    closed_viewers: impl Iterator<Item = i32>,
) {
    let mut viewers = viewers.borrow_mut();

    for idx in closed_viewers {
        info!("removed closed viewer");

        let removed_viewer = viewers.remove(&idx);

        // we know the viewer we just removed
        // was present in the HashMap
        assert!(removed_viewer.is_some())
    }
}

fn nickname_handler(nickname_event_rx: Receiver<NicknameEvent>) {
    let span = span!(Level::INFO, "handling_nicknames");
    let _guard = span.enter();

    let mut taken_nicknames = HashSet::new();

    for nickname_event in nickname_event_rx {
        match nickname_event {
            NicknameEvent::Login {
                nickname,
                is_taken_tx,
            } => handle_login(&mut taken_nicknames, nickname, is_taken_tx),

            NicknameEvent::Logout { ref nickname } => handle_logout(&mut taken_nicknames, nickname),
        }
    }
}

fn handle_login(
    taken_nicknames: &mut HashSet<String>,
    nickname: String,
    is_taken_tx: Sender<bool>,
) {
    info!("received login");

    let is_nickname_taken = !taken_nicknames.insert(nickname);

    if is_nickname_taken {
        info!("nickname was taken");
    } else {
        info!("nickname was not taken");
    }

    is_taken_tx.send(is_nickname_taken).unwrap();
}

fn handle_logout(taken_nicknames: &mut HashSet<String>, nickname: &str) {
    info!("received logout");

    let was_taken = taken_nicknames.remove(nickname);
    assert!(was_taken);
}

enum NicknameEvent {
    Login {
        nickname: String,
        is_taken_tx: Sender<bool>,
    },
    Logout {
        nickname: String,
    },
}
