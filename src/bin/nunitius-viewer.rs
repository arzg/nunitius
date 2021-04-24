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

    let (ui_event_tx, ui_event_rx) = flume::unbounded();

    thread::spawn(|| {
        if let Err(e) = listen_for_ui_events(ui_event_tx) {
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
            .recv(&ui_event_rx, |ui_event| {
                let mut timeline = timeline.borrow_mut();

                match ui_event.unwrap() {
                    UiEvent::Up => timeline.scroll_up(),
                    UiEvent::Down => timeline.scroll_down(),

                    // we leave one line free for currently typing users
                    UiEvent::Resize { height } => timeline.resize(height - 1),

                    UiEvent::Quit => return ControlFlow::Break,
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

enum UiEvent {
    Up,
    Down,
    Resize { height: usize },
    Quit,
}

fn listen_for_ui_events(ui_event_tx: Sender<UiEvent>) -> anyhow::Result<()> {
    loop {
        match event::read()? {
            event::Event::Key(event::KeyEvent {
                code, modifiers, ..
            }) => match (code, modifiers) {
                (event::KeyCode::Char('c'), event::KeyModifiers::CONTROL) => {
                    ui_event_tx.send(UiEvent::Quit).unwrap();
                    break;
                }
                (event::KeyCode::Up, _) => ui_event_tx.send(UiEvent::Up).unwrap(),
                (event::KeyCode::Down, _) => ui_event_tx.send(UiEvent::Down).unwrap(),
                _ => {}
            },

            event::Event::Resize(_, height) => {
                let height = usize::from(height);
                ui_event_tx.send(UiEvent::Resize { height }).unwrap();
            }

            _ => {}
        }
    }

    Ok(())
}
