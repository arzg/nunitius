use flume::{Receiver, Selector, Sender};
use nunitius::{ConnectionKind, Event, Login, LoginResponse};
use std::cell::RefCell;
use std::collections::HashSet;
use std::net::{TcpListener, TcpStream};
use std::{io, thread};

fn main() -> anyhow::Result<()> {
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
            if let Err(e) = handle_connection(stream, viewer_tx, events_tx, nickname_event_tx) {
                eprintln!("Error: {}", e);
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

    match connection_kind {
        ConnectionKind::Sender => {
            let mut connection = jsonl::Connection::new_from_tcp_stream(stream)?;

            let nickname = loop {
                let login: Login = connection.read()?;

                let is_nickname_taken = {
                    let (is_nickname_taken_tx, is_nickname_taken_rx) = flume::bounded(0);

                    nickname_event_tx.send(NicknameEvent::Login {
                        nickname: login.nickname.clone(),
                        is_taken_tx: is_nickname_taken_tx,
                    })?;

                    is_nickname_taken_rx.recv().unwrap()
                };

                connection.write(&LoginResponse {
                    nickname_taken: is_nickname_taken,
                })?;

                if !is_nickname_taken {
                    events_tx.send(Event::Login(login.clone())).unwrap();
                    break login.nickname;
                }
            };

            loop {
                match connection.read() {
                    Ok(message) => events_tx.send(Event::Message(message)).unwrap(),

                    Err(jsonl::ReadError::Eof) => {
                        nickname_event_tx
                            .send(NicknameEvent::Logout { nickname })
                            .unwrap();
                        break;
                    }

                    Err(e) => return Err(e.into()),
                }
            }
        }
        ConnectionKind::Viewer => viewer_tx.send(stream).unwrap(),
    }

    Ok(())
}

fn viewer_handler(events_rx: Receiver<Event>, viewer_rx: Receiver<TcpStream>) {
    let viewers = RefCell::new(Vec::new());

    loop {
        Selector::new()
            .recv(&viewer_rx, |viewer| {
                viewers.borrow_mut().push(viewer.unwrap());
            })
            .recv(&events_rx, |event| {
                let event = event.unwrap();

                let mut closed_viewers = Vec::new();
                let mut viewers = viewers.borrow_mut();

                for (idx, viewer) in viewers.iter_mut().enumerate() {
                    match jsonl::write(viewer, &event) {
                        Ok(()) => {}

                        Err(jsonl::WriteError::Io(io_error))
                            if io_error.kind() == io::ErrorKind::BrokenPipe =>
                        {
                            closed_viewers.push(idx);
                        }

                        Err(e) => eprintln!("Error: {}", anyhow::Error::new(e)),
                    }
                }

                for idx in closed_viewers {
                    viewers.remove(idx);
                }
            })
            .wait();
    }
}

fn nickname_handler(nickname_event_rx: Receiver<NicknameEvent>) {
    let mut taken_nicknames = HashSet::new();

    for nickname_event in nickname_event_rx {
        match nickname_event {
            NicknameEvent::Login {
                nickname,
                is_taken_tx,
            } => {
                let is_nickname_taken = !taken_nicknames.insert(nickname);
                is_taken_tx.send(is_nickname_taken).unwrap();
            }

            NicknameEvent::Logout { nickname } => {
                let was_taken = taken_nicknames.remove(&nickname);

                // panic if the nickname we were told is logging out
                // was not taken in the first place
                assert!(was_taken);
            }
        }
    }
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
