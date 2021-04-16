use ansi_term::{Color as AnsiColor, Style};
use core::fmt;
use nunitius::{Color, ConnectionKind, Event, Message, User};
use std::io::BufReader;
use std::net::TcpStream;

fn main() -> anyhow::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:9999")?;

    jsonl::write(&mut stream, &ConnectionKind::Viewer)?;

    let mut stream = BufReader::new(stream);

    let existing_events: Vec<_> = jsonl::read(&mut stream)?;

    for event in existing_events {
        display_event(event);
    }

    loop {
        let event = jsonl::read(&mut stream)?;
        display_event(event);
    }
}

fn display_event(event: Event) {
    match event {
        Event::Message(Message { body, author }) => println!("{}: {}", format_user(author), body),
        Event::Login(user) => println!("{} logged in!", format_user(user)),
        Event::Logout(user) => println!("{} logged out!", format_user(user)),
    }
}

fn format_user(user: User) -> impl fmt::Display {
    let style = Style::new().bold();

    let style = match user.color {
        Some(Color::Red) => style.fg(AnsiColor::Red),
        Some(Color::Green) => style.fg(AnsiColor::Green),
        Some(Color::Yellow) => style.fg(AnsiColor::Yellow),
        Some(Color::Blue) => style.fg(AnsiColor::Blue),
        Some(Color::Purple) => style.fg(AnsiColor::Purple),
        Some(Color::Cyan) => style.fg(AnsiColor::Cyan),
        None => style,
    };

    style.paint(user.nickname)
}
