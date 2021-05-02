mod editor;

use crossterm::{cursor, event, queue, terminal};
use std::convert::TryInto;
use std::io::{self, Write};

fn main() -> anyhow::Result<()> {
    terminal::enable_raw_mode()?;

    let mut stdout = io::stdout();

    let (terminal_width, _) = terminal::size()?;
    let mut editor = editor::Editor::new((terminal_width - 1).into());

    loop {
        render(&editor, &mut stdout)?;

        let event = event::read()?;
        match event {
            event::Event::Key(event::KeyEvent { code, modifiers }) => match (code, modifiers) {
                (event::KeyCode::Char('c'), event::KeyModifiers::CONTROL) => break,
                (event::KeyCode::Char(c), _) => editor.add(c),
                (event::KeyCode::Backspace, _) => editor.backspace(),
                (event::KeyCode::Enter, _) => editor.enter(),
                (event::KeyCode::Up, _) => editor.move_up(),
                (event::KeyCode::Down, _) => editor.move_down(),
                (event::KeyCode::Left, _) => editor.move_left(),
                (event::KeyCode::Right, _) => editor.move_right(),
                _ => {}
            },
            event::Event::Resize(new_width, _) => editor.resize((new_width - 1).into()),
            _ => {}
        }
    }

    terminal::disable_raw_mode()?;

    Ok(())
}

fn render(editor: &editor::Editor, stdout: &mut io::Stdout) -> anyhow::Result<()> {
    queue!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )?;

    write!(
        stdout,
        "{}",
        editor.render().lines().collect::<Vec<_>>().join("\r\n")
    )?;

    let (line, column) = editor.cursor();
    let line = line.try_into().unwrap();
    let column = column.try_into().unwrap();
    queue!(stdout, cursor::MoveTo(column, line))?;

    stdout.flush()?;

    Ok(())
}
