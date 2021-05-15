mod editor;
mod text_field;

use crossterm::{cursor, event, queue, terminal};
use std::convert::TryInto;
use std::env;
use std::io::{self, Write};

fn main() -> anyhow::Result<()> {
    if let Some(subcommand) = env::args().nth(1) {
        let input = match subcommand.as_str() {
            "text_field" => run_text_field()?,
            "editor" => run_editor()?,
            _ => anyhow::bail!("must specify either ‘text_field’ or ‘editor’ subcommand"),
        };

        queue!(
            io::stdout(),
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        println!("You typed: ‘{}’", input);
    } else {
        anyhow::bail!("must specify either ‘text_field’ or ‘editor’ subcommand");
    }

    Ok(())
}

fn run_text_field() -> anyhow::Result<String> {
    terminal::enable_raw_mode()?;

    let mut stdout = io::stdout();

    let (terminal_width, _) = terminal::size()?;
    let mut text_field = text_field::TextField::new((terminal_width - 1).into());

    loop {
        render_text_field(&text_field, &mut stdout)?;

        let event = event::read()?;
        match event {
            event::Event::Key(event::KeyEvent { code, modifiers }) => match (code, modifiers) {
                (event::KeyCode::Char('c'), event::KeyModifiers::CONTROL)
                | (event::KeyCode::Enter, _) => break,
                (event::KeyCode::Char(c), _) => text_field.add(&c.to_string()),
                (event::KeyCode::Backspace, _) => text_field.backspace(),
                (event::KeyCode::Up, _) => text_field.move_up(),
                (event::KeyCode::Down, _) => text_field.move_down(),
                (event::KeyCode::Left, _) => text_field.move_left(),
                (event::KeyCode::Right, _) => text_field.move_right(),
                _ => {}
            },
            event::Event::Resize(new_width, _) => text_field.resize((new_width - 1).into()),

            _ => {}
        }
    }

    terminal::disable_raw_mode()?;

    Ok(text_field.contents().to_string())
}

fn render_text_field(
    text_field: &text_field::TextField,
    stdout: &mut io::Stdout,
) -> anyhow::Result<()> {
    queue!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )?;

    write!(stdout, "{}", text_field.render())?;

    let column = text_field.cursor();
    let column = column.try_into().unwrap();
    queue!(stdout, cursor::MoveTo(column, 0))?;

    stdout.flush()?;

    Ok(())
}

fn run_editor() -> anyhow::Result<String> {
    terminal::enable_raw_mode()?;

    let mut stdout = io::stdout();

    let (terminal_width, terminal_height) = terminal::size()?;
    let mut editor = editor::Editor::new((terminal_width - 1).into(), terminal_height.into());

    loop {
        render_editor(&editor, &mut stdout)?;

        let event = event::read()?;
        match event {
            event::Event::Key(event::KeyEvent { code, modifiers }) => match (code, modifiers) {
                (event::KeyCode::Char('c'), event::KeyModifiers::CONTROL) => break,
                (event::KeyCode::Char(c), _) => editor.add(&c.to_string()),
                (event::KeyCode::Backspace, _) => editor.backspace(),
                (event::KeyCode::Enter, _) => editor.enter(),
                (event::KeyCode::Up, _) => editor.move_up(),
                (event::KeyCode::Down, _) => editor.move_down(),
                (event::KeyCode::Left, _) => editor.move_left(),
                (event::KeyCode::Right, _) => editor.move_right(),
                _ => {}
            },
            event::Event::Resize(new_width, new_height) => {
                editor.resize_width((new_width - 1).into());
                editor.resize_height(new_height.into());
            }
            _ => {}
        }
    }

    terminal::disable_raw_mode()?;

    Ok(editor.contents())
}

fn render_editor(editor: &editor::Editor, stdout: &mut io::Stdout) -> anyhow::Result<()> {
    queue!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )?;

    write!(stdout, "{}", editor.render().join("\r\n"))?;

    let (line, column) = editor.cursor();
    let line = line.try_into().unwrap();
    let column = column.try_into().unwrap();
    queue!(stdout, cursor::MoveTo(column, line))?;

    stdout.flush()?;

    Ok(())
}
