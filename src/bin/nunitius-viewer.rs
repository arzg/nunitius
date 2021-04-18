use chrono::Local;
use crossterm::style::{self, style, Styler};
use crossterm::{cursor, queue, terminal};
use nunitius::{Color, ConnectionKind, Event, EventKind, Message, TypingEvent, User};
use std::fmt;
use std::io::BufReader;
use std::io::{self, Write};
use std::net::TcpStream;

fn main() -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let mut stream = TcpStream::connect("127.0.0.1:9999")?;

    jsonl::write(&mut stream, &ConnectionKind::Viewer)?;

    let mut stream = BufReader::new(stream);

    let history: Vec<_> = jsonl::read(&mut stream)?;
    let mut events = history;

    loop {
        queue!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
        )?;

        for event in &events {
            display_event(event, &mut stdout)?;
        }

        let event = jsonl::read(&mut stream)?;
        events.push(event);
    }
}

fn display_event(
    Event {
        event,
        user,
        time_occurred,
    }: &Event,
    stdout: &mut io::Stdout,
) -> anyhow::Result<()> {
    let user = format_user(user);

    let local_time_occurred = time_occurred.with_timezone(&Local);
    write!(stdout, "[{}] ", local_time_occurred.format("%H:%M"))?;

    match event {
        EventKind::Message(Message { body }) => {
            writeln!(stdout, "{}: {}", user, body)?;
        }
        EventKind::Login => writeln!(stdout, "{} logged in!", user)?,
        EventKind::Logout => writeln!(stdout, "{} logged out!", user)?,
        EventKind::Typing(event) => match event {
            TypingEvent::Start => writeln!(stdout, "{} started typing...", user)?,
            TypingEvent::Stop => writeln!(stdout, "{} stopped typing...", user)?,
        },
    }

    Ok(())
}

fn format_user(user: &User) -> impl fmt::Display + '_ {
    let base_styled_content = style(&user.nickname).bold();

    if let Some(ref color) = user.color {
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
