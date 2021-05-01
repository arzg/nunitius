#[derive(Debug, Default)]
pub(crate) struct Editor {
    buffer: Vec<char>,
    cursor: usize,
}

impl Editor {
    pub(crate) fn render(&self, width: usize) -> String {
        let text: String = self.buffer.iter().collect();
        textwrap::fill(&text, width)
    }

    pub(crate) fn cursor(&self) -> (usize, usize) {
        (0, self.cursor)
    }

    pub(crate) fn add(&mut self, c: char) {
        if self.at_end_of_buffer() {
            self.buffer.push(c);
        } else {
            self.buffer.insert(self.cursor, c);
        }

        self.cursor += 1;
    }

    pub(crate) fn backspace(&mut self) {
        if self.at_start_of_buffer() {
            return;
        }

        self.buffer.remove(self.cursor - 1);
        self.cursor -= 1;
    }

    pub(crate) fn move_left(&mut self) {
        if !self.at_start_of_buffer() {
            self.cursor -= 1;
        }
    }

    pub(crate) fn move_right(&mut self) {
        if !self.at_end_of_buffer() {
            self.cursor += 1;
        }
    }

    fn at_start_of_buffer(&mut self) -> bool {
        self.cursor == 0
    }

    fn at_end_of_buffer(&mut self) -> bool {
        self.cursor == self.buffer.len()
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
    fn backspace() {
        let mut editor = Editor::default();

        editor.add('a');
        editor.backspace();

        assert_eq!(editor.render(10), "");
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
    fn move_cursor_left_at_start_of_buffer() {
        let mut editor = Editor::default();

        editor.move_left();

        assert_eq!(editor.cursor(), (0, 0));
    }

    #[test]
    fn move_cursor_right_at_end_of_buffer() {
        let mut editor = Editor::default();

        editor.move_right();

        assert_eq!(editor.cursor(), (0, 0));
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
    fn wrap_text_if_over_width_limit() {
        let mut editor = Editor::default();

        for c in "foo bar baz".chars() {
            editor.add(c);
        }

        assert_eq!(editor.render(7), "foo bar\nbaz");
    }
}
