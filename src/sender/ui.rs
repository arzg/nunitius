use crate::TypingEvent;
use crossterm::{event, queue, terminal};
use flume::{RecvTimeoutError, Sender};
use std::io::{self, Write};
use std::time::Duration;
use std::{fmt, thread};

pub fn read_input(prompt: &str, stdout: &mut io::Stdout) -> anyhow::Result<Option<String>> {
    let (typing_event_tx, _typing_event_rx) = flume::unbounded();
    read_input_evented(prompt, stdout, typing_event_tx, |_, _| Ok(()))
}

pub fn read_input_evented(
    prompt: &str,
    stdout: &mut io::Stdout,
    typing_event_tx: Sender<TypingEvent>,
    mut unknown_key_event_handler: impl FnMut(event::KeyCode, event::KeyModifiers) -> anyhow::Result<()>,
) -> anyhow::Result<Option<String>> {
    let (pressed_key_tx, pressed_key_rx) = flume::bounded(0);

    let handle = thread::spawn(move || {
        let mut current_state = TypingEvent::Stop;
        let mut finished = false;

        while !finished {
            let new_state = match pressed_key_rx.recv_timeout(Duration::from_millis(1000)) {
                Ok(()) => {
                    // a key was pressed before the timeout
                    TypingEvent::Start
                }
                Err(RecvTimeoutError::Timeout) => {
                    // no key was pressed before the timeout
                    TypingEvent::Stop
                }
                Err(RecvTimeoutError::Disconnected) => {
                    // the line has been read,
                    // which means the user cannot type anything more,
                    // so we send one more ‘stopped typing’ event
                    // before returning
                    finished = true;
                    TypingEvent::Stop
                }
            };

            if new_state != current_state {
                current_state = new_state;
                typing_event_tx.send(current_state).unwrap();
            }
        }
    });

    let mut edit_buffer = EditBuffer::default();

    terminal::enable_raw_mode()?;

    loop {
        print_line(prompt, &edit_buffer, stdout)?;

        let event::KeyEvent { code, modifiers } =
            if let event::Event::Key(key_event) = event::read()? {
                key_event
            } else {
                continue;
            };

        match (code, modifiers) {
            (event::KeyCode::Char('c'), event::KeyModifiers::CONTROL) => {
                terminal::disable_raw_mode()?;
                std::process::exit(1);
            }
            (event::KeyCode::Char(c), event::KeyModifiers::SHIFT) => {
                edit_buffer.add_multiple(c.to_uppercase());
            }
            (event::KeyCode::Char(c), event::KeyModifiers::NONE) => edit_buffer.add(c),
            (event::KeyCode::Enter, _) => {
                drop(pressed_key_tx);
                handle.join().unwrap();
                break;
            }
            (event::KeyCode::Backspace, _) => edit_buffer.backspace(),
            _ => {
                unknown_key_event_handler(code, modifiers)?;
                continue;
            }
        }

        pressed_key_tx.send(()).unwrap();
    }

    terminal::disable_raw_mode()?;
    writeln!(stdout)?;

    let s = edit_buffer.to_string();
    let s = s.trim().to_string();

    Ok(if s.is_empty() { None } else { Some(s) })
}

#[derive(Default)]
struct EditBuffer {
    buffer: Vec<char>,
}

impl EditBuffer {
    fn add(&mut self, c: char) {
        self.buffer.push(c);
    }

    fn add_multiple(&mut self, cs: impl Iterator<Item = char>) {
        self.buffer.extend(cs);
    }

    fn backspace(&mut self) {
        self.buffer.pop();
    }
}

impl fmt::Display for EditBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in &self.buffer {
            write!(f, "{}", c)?;
        }

        Ok(())
    }
}

fn print_line(
    prompt: &str,
    current_input: &impl fmt::Display,
    stdout: &mut io::Stdout,
) -> anyhow::Result<()> {
    queue!(stdout, terminal::Clear(terminal::ClearType::CurrentLine))?;
    write!(stdout, "\r{} > {}", prompt, current_input)?;
    stdout.flush()?;

    Ok(())
}
