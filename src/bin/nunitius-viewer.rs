use chrono::Local;
use crossterm::style::{self, style, Styler};
use crossterm::{cursor, event, queue, terminal};
use flume::{Selector, Sender};
use nunitius::{Color, ConnectionKind, Event, EventKind, Message, TypingEvent, User};
use std::collections::HashSet;
use std::io::BufReader;
use std::io::{self, Write};
use std::net::TcpStream;
use std::{fmt, thread};

fn main() -> anyhow::Result<()> {
    terminal::enable_raw_mode()?;

    let mut stdout = io::stdout();
    let mut stream = TcpStream::connect("127.0.0.1:9999")?;

    jsonl::write(&mut stream, &ConnectionKind::Viewer)?;

    let mut stream = BufReader::new(stream);

    let history: Vec<_> = jsonl::read(&mut stream)?;
    let mut events = history;
    let mut currently_typing_users = HashSet::new();

    let (input_tx, input_rx) = flume::unbounded();
    let (event_tx, event_rx) = flume::unbounded();

    thread::spawn(|| {
        if let Err(e) = listen_for_input(input_tx) {
            eprintln!("Error: {:#}", e);
        }
    });

    thread::spawn(move || {
        if let Err(e) = listen_for_events(&mut stream, event_tx) {
            eprintln!("Error: {:#}", e);
        }
    });

    loop {
        queue!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
        )?;

        let (_, num_terminal_rows) = terminal::size()?;

        let events_without_typing_events: Vec<_> = events
            .iter()
            .filter(|Event { event, .. }| !matches!(event, EventKind::Typing(_)))
            .collect();

        for event in &events_without_typing_events[events_without_typing_events
            .len()
            .saturating_sub(num_terminal_rows as usize)..]
        {
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

        let control_flow = Selector::new()
            .recv(&event_rx, |event| {
                let event = event.unwrap();

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

                ControlFlow::Continue
            })
            .recv(&input_rx, |input| {
                let input = input.unwrap();

                match input {
                    Input::Up => {}
                    Input::Down => {}
                    Input::Quit => return ControlFlow::Break,
                }

                ControlFlow::Continue
            })
            .wait();

        if let ControlFlow::Break = control_flow {
            break;
        }
    }

    terminal::disable_raw_mode()?;

    Ok(())
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
            write!(stdout, "[{}] {}: {}", local_time_occurred, user, body)?;
        }
        EventKind::Login => write!(stdout, "[{}] {} logged in!", local_time_occurred, user)?,
        EventKind::Logout => write!(stdout, "[{}] {} logged out!", local_time_occurred, user)?,
        EventKind::Typing(_) => return Ok(()),
    }

    writeln!(stdout, "\r")?;

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

enum ControlFlow {
    Continue,
    Break,
}

fn listen_for_events(
    stream: &mut BufReader<TcpStream>,
    event_tx: Sender<Event>,
) -> anyhow::Result<()> {
    loop {
        let event = jsonl::read(&mut *stream)?;
        event_tx.send(event).unwrap();
    }
}

enum Input {
    Up,
    Down,
    Quit,
}

fn listen_for_input(input_tx: Sender<Input>) -> anyhow::Result<()> {
    loop {
        if let event::Event::Key(event::KeyEvent {
            code, modifiers, ..
        }) = event::read()?
        {
            match (code, modifiers) {
                (event::KeyCode::Char('c'), event::KeyModifiers::CONTROL) => {
                    input_tx.send(Input::Quit).unwrap();
                    break;
                }
                (event::KeyCode::Up, _) => input_tx.send(Input::Up).unwrap(),
                (event::KeyCode::Down, _) => input_tx.send(Input::Down).unwrap(),
                _ => {}
            }
        }
    }

    Ok(())
}
