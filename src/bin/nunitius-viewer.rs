use crossterm::style::{self, style, Styler};
use nunitius::{Color, ConnectionKind, Event, Message, TypingEvent, User};
use std::fmt;
use std::io::BufReader;
use std::io::{self, Write};
use std::net::TcpStream;

fn main() -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let mut stream = TcpStream::connect("127.0.0.1:9999")?;

    jsonl::write(&mut stream, &ConnectionKind::Viewer)?;

    let mut stream = BufReader::new(stream);

    let existing_events: Vec<_> = jsonl::read(&mut stream)?;

    for event in existing_events {
        display_event(event, &mut stdout)?;
    }

    loop {
        let event = jsonl::read(&mut stream)?;
        display_event(event, &mut stdout)?;
    }
}

fn display_event(event: Event, stdout: &mut io::Stdout) -> anyhow::Result<()> {
    match event {
        Event::Message(Message { body, author }) => {
            writeln!(stdout, "{}: {}", display_user(author), body)?
        }
        Event::Login(user) => writeln!(stdout, "{} logged in!", display_user(user))?,
        Event::Logout(user) => writeln!(stdout, "{} logged out!", display_user(user))?,
        Event::Typing { event, user } => match event {
            TypingEvent::Start => writeln!(stdout, "{} started typing...", display_user(user))?,
            TypingEvent::Stop => writeln!(stdout, "{} stopped typing...", display_user(user))?,
        },
    }

    Ok(())
}

fn display_user(user: User) -> impl fmt::Display {
    let base_styled_content = style(user.nickname).bold();

    if let Some(color) = user.color {
        let color = match color {
            Color::Red => style::Color::Red,
            Color::Green => style::Color::Green,
            Color::Yellow => style::Color::Yellow,
            Color::Blue => style::Color::Blue,
            Color::Magenta => style::Color::Magenta,
            Color::Cyan => style::Color::Cyan,
        };

        base_styled_content.with(color)
    } else {
        base_styled_content
    }
}
