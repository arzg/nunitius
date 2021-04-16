use super::NicknameEvent;
use crate::{Event, Login, LoginResponse};
use flume::{Receiver, Sender};
use log::{error, info};
use std::net::TcpStream;
use std::{io, thread};

pub fn sender_handler(
    sender_rx: Receiver<TcpStream>,
    nickname_event_tx: Sender<NicknameEvent>,
    event_tx: Sender<Event>,
) {
    for stream in sender_rx {
        info!("received new sender");
        let nickname_event_tx = nickname_event_tx.clone();
        let event_tx = event_tx.clone();

        thread::spawn(|| {
            if let Err(e) = handle_sender(stream, nickname_event_tx, event_tx) {
                error!("{:#}", e);
            }
        });
    }
}

fn handle_sender(
    stream: TcpStream,
    nickname_event_tx: Sender<NicknameEvent>,
    event_tx: Sender<Event>,
) -> anyhow::Result<()> {
    let mut connection = jsonl::Connection::new_from_tcp_stream(stream)?;
    let nickname = log_sender_in(&mut connection, &nickname_event_tx, &event_tx)?;

    loop {
        match connection.read() {
            Ok(message) => {
                info!("received message");
                event_tx.send(Event::Message(message)).unwrap();
            }

            Err(jsonl::ReadError::Eof) => {
                info!("logged out");

                nickname_event_tx
                    .send(NicknameEvent::Logout {
                        nickname: nickname.clone(),
                    })
                    .unwrap();

                event_tx.send(Event::Logout { nickname }).unwrap();

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
    event_tx: &Sender<Event>,
) -> anyhow::Result<String> {
    loop {
        let login: Login = connection.read()?;
        info!("read login from sender: {:?}", login);

        let is_nickname_taken =
            check_if_nickname_is_taken(login.nickname.clone(), nickname_event_tx)?;

        connection.write(&LoginResponse {
            nickname_taken: is_nickname_taken,
        })?;

        if is_nickname_taken {
            info!("nickname was taken, retrying");
        } else {
            info!("logged in with unique nickname");
            event_tx.send(Event::Login(login.clone())).unwrap();

            return Ok(login.nickname);
        }
    }
}

fn check_if_nickname_is_taken(
    nickname: String,
    nickname_event_tx: &Sender<NicknameEvent>,
) -> anyhow::Result<bool> {
    let (is_nickname_taken_tx, is_nickname_taken_rx) = flume::bounded(0);

    nickname_event_tx.send(NicknameEvent::Login {
        nickname,
        is_taken_tx: is_nickname_taken_tx,
    })?;

    Ok(is_nickname_taken_rx.recv().unwrap())
}

type TcpConnection = jsonl::Connection<io::BufReader<TcpStream>, TcpStream>;
