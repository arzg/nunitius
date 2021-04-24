use crossterm::{cursor, event, queue, terminal};
use flume::{Selector, Sender};
use nunitius::viewer::{Protocol, Timeline};
use nunitius::{Event as ServerEvent, EventKind as ServerEventKind, TypingEvent};
use std::cell::RefCell;
use std::collections::HashSet;
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

    let timeline = {
        let (_, num_terminal_rows) = terminal::size()?;

        // we leave one line free for currently typing users
        RefCell::new(Timeline::new(usize::from(num_terminal_rows) - 1))
    };

    let mut currently_typing_users = HashSet::new();

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

        for event in timeline.borrow().visible_events() {
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
                timeline.borrow_mut().add_event(event.unwrap());
                ControlFlow::Continue
            })
            .recv(&input_rx, |input| {
                match input.unwrap() {
                    Input::Up => timeline.borrow_mut().move_up(),
                    Input::Down => timeline.borrow_mut().move_down(),
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
