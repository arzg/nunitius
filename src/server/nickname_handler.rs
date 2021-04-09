use super::NicknameEvent;
use flume::{Receiver, Sender};
use std::collections::HashSet;
use tracing::{info, span, Level};

pub fn nickname_handler(nickname_event_rx: Receiver<NicknameEvent>) {
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
