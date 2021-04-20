use crossterm::{cursor, event, queue, terminal};
use flume::{Selector, Sender};
use nunitius::viewer::Protocol;
use nunitius::{Event as ServerEvent, EventKind as ServerEventKind, TypingEvent};
use std::cell::Cell;
use std::collections::HashSet;
use std::convert::TryInto;
use std::io::{self, Write};
use std::thread;

fn main() -> anyhow::Result<()> {
    terminal::enable_raw_mode()?;

    let mut stdout = io::stdout();
    let protocol = Protocol::connect("127.0.0.1:9999")?;

    let (server_event_tx, server_event_rx) = flume::bounded(100);
    let (event_tx, event_rx) = flume::bounded(100);

    let protocol = protocol.send_connection_kind(server_event_tx, event_tx)?;
    let mut protocol = protocol.read_history()?;

    let mut events = Vec::new();
    let mut currently_typing_users = HashSet::new();
    let cursor_position = Cell::new(0);

    let (input_tx, input_rx) = flume::unbounded();

    thread::spawn(|| {
        if let Err(e) = listen_for_input(input_tx) {
            eprintln!("Error: {:#}", e);
        }
    });

    thread::spawn(move || {
        if let Err(e) = protocol.read_events() {
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

        let start_idx = ((events.len().saturating_sub(num_terminal_rows as usize) as isize)
            + cursor_position.get())
        .max(0);

        let end_idx = ((events.len() as isize) + cursor_position.get())
            .max(start_idx + num_terminal_rows as isize)
            .min(events.len() as isize);

        for event in &events[start_idx.try_into().unwrap()..end_idx.try_into().unwrap()] {
            writeln!(stdout, "{}\r", nunitius::viewer::render_event(event))?;
        }

        if !currently_typing_users.is_empty() {
            let (_, num_terminal_rows) = terminal::size()?;
            queue!(stdout, cursor::MoveTo(0, num_terminal_rows - 1))?;

            write!(
                stdout,
                "{}",
                nunitius::viewer::render_currently_typing_users(currently_typing_users.iter()),
            )?;
        }
        stdout.flush()?;

        let control_flow = Selector::new()
            .recv(&server_event_rx, |server_event| {
                let server_event = server_event.unwrap();

                if let ServerEvent {
                    event: ServerEventKind::Typing(typing_event),
                    ref user,
                    ..
                } = server_event
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

                ControlFlow::Continue
            })
            .recv(&event_rx, |event| {
                let event = event.unwrap();

                cursor_position.set(0);
                events.push(event);

                ControlFlow::Continue
            })
            .recv(&input_rx, |input| {
                let input = input.unwrap();

                match input {
                    Input::Up => cursor_position.set(cursor_position.get() - 1),

                    Input::Down => {
                        cursor_position.set(cursor_position.get() + 1);
                        cursor_position.set(cursor_position.get().min(0));
                    }
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

enum ControlFlow {
    Continue,
    Break,
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
