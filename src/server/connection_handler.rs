use super::NicknameEvent;
use crate::{ConnectionKind, Event, Login, LoginResponse};
use flume::Sender;
use std::io;
use std::net::TcpStream;
use tracing::info;

pub fn handle_connection(
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

type TcpConnection = jsonl::Connection<io::BufReader<TcpStream>, TcpStream>;
