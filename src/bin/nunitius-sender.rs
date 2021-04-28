use crossterm::{cursor, event, execute, terminal};
use flume::Sender;
use jsonl::Connection;
use nunitius::sender::ui;
use nunitius::{
    Color, ConnectionKind, Login, LoginResponse, Message, SenderEvent, TypingEvent, User,
};
use std::io::{self, Write};
use std::net::TcpStream;
use std::{fs, thread};

type TcpConnection = Connection<io::BufReader<TcpStream>, TcpStream>;

fn main() -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    let stream = TcpStream::connect("127.0.0.1:9999")?;
    let mut connection = Connection::new_from_tcp_stream(stream)?;

    connection.write(&ConnectionKind::Sender)?;

    login(&mut connection, &mut stdout, &mut stderr)?;

    let (typing_event_tx, typing_event_rx) = flume::bounded(100);
    let (sender_event_tx, sender_event_rx) = flume::bounded(100);

    thread::spawn({
        let sender_event_tx = sender_event_tx.clone();

        move || {
            for typing_event in typing_event_rx {
                sender_event_tx
                    .send(SenderEvent::Typing(typing_event))
                    .unwrap();
            }
        }
    });

    thread::spawn(move || {
        for sender_event in sender_event_rx {
            connection.write(&sender_event).unwrap();
        }
    });

    loop {
        let input = read_and_clear_evented(
            "Type a message",
            &mut io::stdout(),
            typing_event_tx.clone(),
            |code, modifiers| {
                if let (event::KeyCode::Char('u'), event::KeyModifiers::CONTROL) = (code, modifiers)
                {
                    handle_file_upload(&mut stdout, &sender_event_tx)?;
                }

                Ok(())
            },
        )?;

        if let Some(input) = input {
            sender_event_tx
                .send(SenderEvent::Message(Message::Text { body: input }))
                .unwrap();
        }
    }
}

fn handle_file_upload(
    stdout: &mut io::Stdout,
    sender_event_tx: &flume::Sender<SenderEvent>,
) -> Result<(), anyhow::Error> {
    loop {
        let path = read_and_clear("Choose a file to upload", stdout)?;

        let path = if let Some(path) = path {
            path
        } else {
            continue;
        };

        let file_contents = fs::read(path)?;

        sender_event_tx
            .send(SenderEvent::Message(Message::File {
                contents: file_contents,
            }))
            .unwrap();

        break;
    }

    execute!(
        stdout,
        cursor::MoveUp(1),
        terminal::Clear(terminal::ClearType::CurrentLine),
    )?;
    // write!(stdout, "\r")?;

    Ok(())
}

fn login(
    connection: &mut TcpConnection,
    stdout: &mut io::Stdout,
    stderr: &mut io::Stderr,
) -> anyhow::Result<()> {
    loop {
        let nickname = read_and_clear("Choose a nickname", stdout)?;

        let nickname = if let Some(n) = nickname {
            n
        } else {
            continue;
        };

        let user = User {
            nickname: nickname.clone(),
            color: read_color(stdout, stderr)?,
        };

        connection.write(&Login { user })?;

        let response: LoginResponse = connection.read()?;

        if response.nickname_taken {
            writeln!(stderr, "Nickname ‘{}’ taken. Try another one.", nickname)?;
        } else {
            return Ok(());
        }
    }
}

fn read_color(stdout: &mut io::Stdout, stderr: &mut io::Stderr) -> anyhow::Result<Option<Color>> {
    loop {
        let color = if let Some(s) = read_and_clear("Choose a color", stdout)? {
            s
        } else {
            return Ok(None);
        };

        let color = match color.as_str() {
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            _ => {
                writeln!(stderr, "‘{}’ is an invalid color.", color)?;
                continue;
            }
        };

        return Ok(Some(color));
    }
}

fn read_and_clear(prompt: &str, stdout: &mut io::Stdout) -> anyhow::Result<Option<String>> {
    let output = ui::read_input(prompt, stdout)?;

    execute!(
        stdout,
        cursor::MoveUp(1),
        terminal::Clear(terminal::ClearType::CurrentLine),
    )?;

    Ok(output)
}

fn read_and_clear_evented(
    prompt: &str,
    stdout: &mut io::Stdout,
    typing_event_tx: Sender<TypingEvent>,
    unknown_key_event_handler: impl FnMut(event::KeyCode, event::KeyModifiers) -> anyhow::Result<()>,
) -> anyhow::Result<Option<String>> {
    let output =
        ui::read_input_evented(prompt, stdout, typing_event_tx, unknown_key_event_handler)?;

    execute!(
        stdout,
        cursor::MoveUp(1),
        terminal::Clear(terminal::ClearType::CurrentLine),
    )?;

    Ok(output)
}
