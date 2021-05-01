#[derive(Debug)]
pub(crate) struct Editor {
    buffer: Vec<String>,
    line: usize,
    column: usize,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            buffer: vec![String::new()],
            line: 0,
            column: 0,
        }
    }
}

impl Editor {
    pub(crate) fn render(&self, width: usize) -> String {
        let text: String = self.buffer.join("\n");
        textwrap::fill(&text, width)
    }

    pub(crate) fn cursor(&self) -> (usize, usize) {
        (self.line, self.column)
    }

    pub(crate) fn add(&mut self, c: char) {
        if self.at_end_of_line() {
            self.buffer[self.line].push(c);
        } else {
            self.buffer[self.line].insert(self.column, c);
        }

        self.column += 1;
    }

    pub(crate) fn backspace(&mut self) {
        if self.at_start_of_line() {
            return;
        }

        self.buffer[self.line].remove(self.column - 1);
        self.column -= 1;
    }

    pub(crate) fn enter(&mut self) {
        let after_cursor = self.buffer[self.line].split_off(self.column);

        // the current line now contains everything before the cursor

        self.buffer.insert(self.line + 1, after_cursor);
        self.line += 1;
        self.column = 0;
    }

    pub(crate) fn move_left(&mut self) {
        if self.at_start_of_buffer() {
            return;
        }

        if self.at_start_of_line() {
            self.line -= 1;
            self.move_to_end_of_line();
            return;
        }

        self.column -= 1;
    }

    pub(crate) fn move_right(&mut self) {
        if self.at_end_of_buffer() {
            return;
        }

        if self.at_end_of_line() {
            self.line += 1;
            self.column = 0;
            return;
        }

        self.column += 1;
    }

    pub(crate) fn move_up(&mut self) {
        if self.at_first_line() {
            self.column = 0;
            return;
        }

        self.line -= 1;
    }

    pub(crate) fn move_down(&mut self) {
        if self.at_last_line() {
            self.move_to_end_of_line();
            return;
        }

        self.line += 1;
    }

    fn move_to_end_of_line(&mut self) {
        self.column = self.buffer[self.line].len();
    }

    fn at_start_of_buffer(&self) -> bool {
        self.at_first_line() && self.at_start_of_line()
    }

    fn at_end_of_buffer(&self) -> bool {
        self.at_last_line() && self.at_end_of_line()
    }

    fn at_start_of_line(&self) -> bool {
        self.column == 0
    }

    fn at_end_of_line(&self) -> bool {
        self.buffer[self.line].len() == self.column
    }

    fn at_first_line(&self) -> bool {
        self.line == 0
    }

    fn at_last_line(&self) -> bool {
        self.line == self.buffer.len() || self.buffer.len() == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_text() {
        let mut editor = Editor::default();

        editor.add('c');

        assert_eq!(editor.render(10), "c");
        assert_eq!(editor.cursor(), (0, 1));
    }

    #[test]
    fn add_text_at_cursor() {
        let mut editor = Editor::default();

        editor.add('b');
        editor.move_left();
        editor.add('a');
        editor.move_right();
        editor.add('c');

        assert_eq!(editor.render(10), "abc");
        assert_eq!(editor.cursor(), (0, 3));
    }

    #[test]
    fn backspace() {
        let mut editor = Editor::default();

        editor.add('a');
        editor.backspace();

        assert_eq!(editor.render(10), "");
        assert_eq!(editor.cursor(), (0, 0));
    }

    #[test]
    fn backspace_at_cursor() {
        let mut editor = Editor::default();

        editor.add('a');
        editor.add('b');
        editor.move_left();
        editor.backspace();

        assert_eq!(editor.render(10), "b");
        assert_eq!(editor.cursor(), (0, 0));
    }

    #[test]
    fn backspace_at_start_of_buffer() {
        let mut editor = Editor::default();
        editor.backspace();
    }

    #[test]
    fn move_cursor_left() {
        let mut editor = Editor::default();

        editor.add('a');
        editor.move_left();

        assert_eq!(editor.cursor(), (0, 0));
    }

    #[test]
    fn move_cursor_right() {
        let mut editor = Editor::default();

        editor.add('a');
        editor.add('b');
        editor.add('c');
        editor.move_left();
        editor.move_left();
        editor.move_right();

        assert_eq!(editor.cursor(), (0, 2));
    }

    #[test]
    fn move_cursor_up() {
        let mut editor = Editor::default();

        editor.enter();
        editor.move_up();

        assert_eq!(editor.cursor(), (0, 0));
    }

    #[test]
    fn move_cursor_down() {
        let mut editor = Editor::default();

        editor.enter();
        editor.enter();
        editor.move_up();
        editor.move_up();
        editor.move_down();

        assert_eq!(editor.cursor(), (1, 0));
    }

    #[test]
    fn cursor_movement_edge_cases() {
        let mut editor = Editor::default();

        editor.move_left();
        assert_eq!(editor.cursor(), (0, 0));

        editor.move_right();
        assert_eq!(editor.cursor(), (0, 0));

        editor.move_up();
        assert_eq!(editor.cursor(), (0, 0));

        editor.move_down();
        assert_eq!(editor.cursor(), (0, 0));
    }

    #[test]
    fn wrap_cursor_around_at_start_of_line() {
        let mut editor = Editor::default();

        editor.add('a');
        editor.enter();
        editor.move_left();

        assert_eq!(editor.cursor(), (0, 1));
    }

    #[test]
    fn wrap_cursor_around_at_end_of_line() {
        let mut editor = Editor::default();

        editor.add('a');
        editor.enter();
        editor.add('b');
        editor.move_up();
        editor.move_right();

        assert_eq!(editor.cursor(), (1, 0));
    }

    #[test]
    fn move_to_start_of_line_if_at_top_when_moving_up() {
        let mut editor = Editor::default();

        editor.add('a');
        editor.add('b');
        editor.move_up();

        assert_eq!(editor.cursor(), (0, 0));
    }

    #[test]
    fn move_to_end_of_line_if_at_bottom_when_moving_down() {
        let mut editor = Editor::default();

        editor.add('a');
        editor.add('b');
        editor.move_left();
        editor.move_down();

        assert_eq!(editor.cursor(), (0, 2));
    }

    #[test]
    fn enter_at_start_of_line() {
        let mut editor = Editor::default();

        editor.add('a');
        editor.move_left();
        editor.enter();
        editor.add('b');

        assert_eq!(editor.render(10), "\nba");
        assert_eq!(editor.cursor(), (1, 1));
    }

    #[test]
    fn enter_at_end_of_line() {
        let mut editor = Editor::default();

        editor.add('a');
        editor.enter();
        editor.add('b');

        assert_eq!(editor.render(10), "a\nb");
        assert_eq!(editor.cursor(), (1, 1));
    }

    #[test]
    fn enter_in_middle_of_line() {
        let mut editor = Editor::default();

        editor.add('a');
        editor.add('b');
        editor.move_left();
        editor.enter();

        assert_eq!(editor.render(10), "a\nb");
        assert_eq!(editor.cursor(), (1, 0));
    }

    #[test]
    fn wrap_text_if_over_width_limit() {
        let mut editor = Editor::default();

        for c in "foo bar baz".chars() {
            editor.add(c);
        }

        assert_eq!(editor.render(7), "foo bar\nbaz");
    }
}
