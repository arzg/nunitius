use chrono::Local;
use crossterm::style::{self, style, Styler};
use crossterm::{cursor, queue, terminal};
use nunitius::{Color, ConnectionKind, Event, EventKind, Message, TypingEvent, User};
use std::collections::HashSet;
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
    let mut currently_typing_users = HashSet::new();

    loop {
        queue!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
        )?;

        for event in &events {
            display_event(event, &mut stdout)?;
        }

        if !currently_typing_users.is_empty() {
            let (_, num_terminal_rows) = terminal::size()?;
            queue!(stdout, cursor::MoveTo(0, num_terminal_rows - 1))?;

            write!(
                stdout,
                "{} {} typing...",
                currently_typing_users
                    .iter()
                    .map(|user| format_user(user).to_string())
                    .collect::<Vec<_>>()
                    .join(" and "),
                if currently_typing_users.len() == 1 {
                    "is"
                } else {
                    "are"
                },
            )?;
        }

        stdout.flush()?;

        let event = jsonl::read(&mut stream)?;

        if let Event {
            event: EventKind::Typing(typing_event),
            ref user,
            ..
        } = event
        {
            match typing_event {
                TypingEvent::Start => {
                    currently_typing_users.insert(user.clone());
                }
                TypingEvent::Stop => {
                    currently_typing_users.remove(user);
                }
            }
        }

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
    let local_time_occurred = local_time_occurred.format("%H:%M");

    match event {
        EventKind::Message(Message { body }) => {
            writeln!(stdout, "[{}] {}: {}", local_time_occurred, user, body)?;
        }
        EventKind::Login => writeln!(stdout, "[{}] {} logged in!", local_time_occurred, user)?,
        EventKind::Logout => writeln!(stdout, "[{}] {} logged out!", local_time_occurred, user)?,
        EventKind::Typing(_) => {}
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
