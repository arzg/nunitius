use crossterm::style::{style, Stylize};
use crossterm::{event, queue, terminal};
use flume::{Receiver, Selector, Sender};
use std::convert::TryInto;
use std::io::Write;
use std::{io, thread};
use ui_types::StyledText;

pub struct Runtime {
    stdout: io::Stdout,
    input_rx: Receiver<view::Input>,
}

impl Runtime {
    pub fn new() -> io::Result<Self> {
        crossterm::terminal::enable_raw_mode()?;

        let (input_tx, input_rx) = flume::unbounded();

        thread::spawn({
            move || {
                Self::listen_for_events(input_tx)
                    .expect("Failed listening for events from terminal")
            }
        });

        Ok(Self {
            stdout: io::stdout(),
            input_rx,
        })
    }

    pub fn run_view_until_death<V: view::View>(
        &mut self,
        extra: V::Extra,
        message_rx: Receiver<V::Message>,
        command_tx: Sender<V::Command>,
    ) -> io::Result<()> {
        let (width, height) = terminal::size()?;
        let view = V::new(width.into(), height.into(), extra);

        ViewRuntime {
            view,
            input_rx: self.input_rx.clone(),
            message_rx,
            command_tx,
            stdout: self.stdout.lock(),
        }
        .run()
    }

    fn listen_for_events(input_tx: Sender<view::Input>) -> io::Result<()> {
        loop {
            let terminal_event = event::read()?;
            let input = match convert_terminal_event_to_input(terminal_event) {
                Some(input) => input,
                None => continue,
            };

            if input_tx.send(input).is_err() {
                break;
            }
        }

        Ok(())
    }
}

struct ViewRuntime<'a, V: view::View> {
    view: V,
    input_rx: Receiver<view::Input>,
    message_rx: Receiver<V::Message>,
    command_tx: Sender<V::Command>,
    stdout: io::StdoutLock<'a>,
}

impl<'a, V: view::View> ViewRuntime<'a, V> {
    fn run(mut self) -> io::Result<()> {
        self.rerender()?;

        while let Some(message) = self.next_message() {
            if let ControlFlow::Break = self.update_view(message) {
                break;
            }

            self.rerender()?;
        }

        Ok(())
    }

    fn update_view(&mut self, message: V::Message) -> ControlFlow {
        match self.view.update(message) {
            view::StateChange::Alive {
                command: Some(command),
            } => {
                if self.command_tx.send(command).is_err() {
                    return ControlFlow::Break;
                }
            }

            view::StateChange::Alive { command: None } => {}

            view::StateChange::Dead => return ControlFlow::Break,
        }

        ControlFlow::Continue
    }

    fn next_message(&self) -> Option<V::Message> {
        Selector::new()
            .recv(&self.input_rx, |input| input.ok().map(V::Message::from))
            .recv(&self.message_rx, |message| message.ok())
            .wait()
    }

    fn rerender(&mut self) -> io::Result<()> {
        let frame = render(&self.view);
        display(frame, &mut self.stdout)?;

        Ok(())
    }
}

enum ControlFlow {
    Continue,
    Break,
}

struct Frame {
    text: String,
    cursor_row: u16,
    cursor_column: u16,
}

fn display(frame: Frame, stdout: &mut io::StdoutLock<'_>) -> io::Result<()> {
    clear(stdout)?;
    stdout.write_all(frame.text.as_bytes())?;
    queue!(
        stdout,
        crossterm::cursor::MoveTo(frame.cursor_column, frame.cursor_row)
    )?;
    stdout.flush()?;

    Ok(())
}

fn clear(stdout: &mut io::StdoutLock<'_>) -> io::Result<()> {
    queue!(
        stdout,
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
        crossterm::cursor::MoveTo(0, 0)
    )
}

fn render(view: &impl view::View) -> Frame {
    let mut text = String::new();

    for line in view.render() {
        match line {
            StyledText::Bold(s) => text.push_str(&style(s).bold().to_string()),
            StyledText::Regular(s) => text.push_str(&style(s).to_string()),
            StyledText::Red(s) => {
                text.push_str(&style(s).with(crossterm::style::Color::Red).to_string())
            }
        }

        text.push_str("\r\n");
    }

    let (row, column) = view.cursor();
    let row = row.try_into().unwrap();
    let column = column.try_into().unwrap();

    Frame {
        text,
        cursor_row: row,
        cursor_column: column,
    }
}

fn convert_terminal_event_to_input(terminal_event: event::Event) -> Option<view::Input> {
    let input = match terminal_event {
        event::Event::Key(event::KeyEvent {
            code: event::KeyCode::Char('c'),
            modifiers: event::KeyModifiers::CONTROL,
        }) => std::process::exit(0),

        event::Event::Key(event::KeyEvent { code, .. }) => match code {
            event::KeyCode::Char(c) => view::Input::Keypresses(c.to_string()),
            event::KeyCode::Backspace => view::Input::Backspace,
            event::KeyCode::Enter => view::Input::Enter,
            event::KeyCode::Left => view::Input::MoveLeft,
            event::KeyCode::Right => view::Input::MoveRight,
            event::KeyCode::Up => view::Input::MoveUp,
            event::KeyCode::Down => view::Input::MoveDown,
            _ => return None,
        },

        event::Event::Resize(columns, rows) => view::Input::Resize {
            width: (columns - 1).into(),
            height: rows.into(),
        },

        _ => return None,
    };

    Some(input)
}
