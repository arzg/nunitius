mod para;
mod wrap;

use para::Paragraph;
use wrap::wrap;

use itertools::Itertools;

#[derive(Debug)]
pub(crate) struct Editor {
    buffer: Vec<Paragraph>,
    para_idx: usize,
    line: usize,
    column: usize,
    width: usize,
}

impl Editor {
    pub(crate) fn new(width: usize) -> Self {
        Self {
            buffer: vec![Paragraph::default()],
            para_idx: 0,
            line: 0,
            column: 0,
            width,
        }
    }

    pub(crate) fn render(&self) -> String {
        self.buffer
            .iter()
            .map(|para| para.lines().join("\n"))
            .join("\n\n")
    }

    pub(crate) fn resize(&mut self, width: usize) {
        self.width = width;
        self.rewrap();
    }

    pub(crate) fn cursor(&self) -> (usize, usize) {
        let num_lines_in_paras_above_cursor: usize = self.buffer[..self.para_idx]
            .iter()
            .map(Paragraph::num_lines)
            .sum();

        // paragraph breaks are rendered one line high
        let num_para_breaks_above_cursor = self.para_idx;

        (
            num_lines_in_paras_above_cursor + num_para_breaks_above_cursor + self.line,
            self.column,
        )
    }

    pub(crate) fn add(&mut self, c: char) {
        self.buffer[self.para_idx].insert(c, self.line, self.column);
        self.column += 1;
        self.rewrap_current_para();
    }

    pub(crate) fn backspace(&mut self) {
        if self.at_start_of_buffer() {
            return;
        }

        if self.at_start_of_para() {
            self.join_paras();
            return;
        }

        if self.at_start_of_line() {
            self.move_up();
            self.move_to_end_of_line();
        }

        self.buffer[self.para_idx].remove(self.line, self.column - 1);
        self.column -= 1;
        self.rewrap_current_para();
    }

    fn join_paras(&mut self) {
        self.para_idx -= 1;
        self.move_to_end_of_para();

        let para = self.buffer.remove(self.para_idx + 1);
        self.buffer[self.para_idx].join(para);

        self.rewrap_current_para();
    }

    pub(crate) fn enter(&mut self) {
        if self.at_start_of_line() {
            self.buffer.insert(self.para_idx, Paragraph::default());
            self.line = 0;
            self.para_idx += 1;
            return;
        }

        let after_cursor = self.buffer[self.para_idx].split_off(self.line, self.column);

        // the current paragraph now contains everything before the cursor

        self.buffer.insert(self.para_idx + 1, after_cursor);
        self.line = 0;
        self.column = 0;
        self.para_idx += 1;

        self.rewrap_current_para();
    }

    pub(crate) fn move_left(&mut self) {
        if self.at_start_of_buffer() {
            return;
        }

        if self.at_start_of_line() {
            self.move_up();
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
            self.move_down();
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

        if self.at_first_line_of_para() {
            self.para_idx -= 1;
            self.move_to_last_line_of_para();
            return;
        }

        self.line -= 1;
        self.clamp();
    }

    pub(crate) fn move_down(&mut self) {
        if self.at_last_line() {
            self.move_to_end_of_line();
            return;
        }

        if self.at_last_line_of_para() {
            self.para_idx += 1;
            self.line = 0;
            return;
        }

        self.line += 1;
        self.clamp();
    }

    fn rewrap(&mut self) {
        let para_cursor_idx = self.save_para_cursor_idx();

        for para in &mut self.buffer {
            para.rewrap(self.width);
        }

        self.restore_para_cursor_pos(para_cursor_idx);
    }

    fn rewrap_current_para(&mut self) {
        let para_cursor_idx = self.save_para_cursor_idx();
        self.buffer[self.para_idx].rewrap(self.width);
        self.restore_para_cursor_pos(para_cursor_idx);
    }

    fn save_para_cursor_idx(&mut self) -> usize {
        self.buffer[self.para_idx].idx_of_coords(self.line, self.column)
    }

    fn restore_para_cursor_pos(&mut self, para_cursor_idx: usize) {
        let (line, column) = self.buffer[self.para_idx].coords_of_idx(para_cursor_idx);
        self.line = line;
        self.column = column;
    }

    fn move_to_end_of_para(&mut self) {
        self.move_to_last_line_of_para();
        self.move_to_end_of_line();
    }

    fn move_to_last_line_of_para(&mut self) {
        self.line = self.buffer[self.para_idx].num_lines() - 1;
    }

    fn move_to_end_of_line(&mut self) {
        self.column = self.buffer[self.para_idx][self.line].len();
    }

    fn clamp(&mut self) {
        self.column = self.column.min(self.buffer[self.para_idx][self.line].len());
    }

    fn at_start_of_buffer(&self) -> bool {
        self.at_first_line() && self.at_start_of_line()
    }

    fn at_end_of_buffer(&self) -> bool {
        self.at_last_line() && self.at_end_of_line()
    }

    fn at_start_of_para(&self) -> bool {
        self.at_first_line_of_para() && self.at_start_of_line()
    }

    fn at_start_of_line(&self) -> bool {
        self.column == 0
    }

    fn at_end_of_line(&self) -> bool {
        self.buffer[self.para_idx][self.line].len() == self.column
    }

    fn at_first_line(&self) -> bool {
        self.at_first_line_of_para() && self.at_first_para()
    }

    fn at_last_line(&self) -> bool {
        self.at_last_line_of_para() && self.at_last_para()
    }

    fn at_first_line_of_para(&self) -> bool {
        self.line == 0
    }

    fn at_last_line_of_para(&self) -> bool {
        self.line == self.buffer[self.para_idx].num_lines() - 1
    }

    fn at_first_para(&self) -> bool {
        self.para_idx == 0
    }

    fn at_last_para(&self) -> bool {
        self.para_idx == self.buffer.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_text() {
        let mut editor = Editor::new(10);

        editor.add('c');

        assert_eq!(editor.render(), "c");
        assert_eq!(editor.cursor(), (0, 1));
    }

    #[test]
    fn add_text_at_cursor() {
        let mut editor = Editor::new(10);

        editor.add('b');
        editor.move_left();
        editor.add('a');
        editor.move_right();
        editor.add('c');

        assert_eq!(editor.render(), "abc");
        assert_eq!(editor.cursor(), (0, 3));
    }

    #[test]
    fn backspace() {
        let mut editor = Editor::new(10);

        editor.add('a');
        editor.backspace();

        assert_eq!(editor.render(), "");
        assert_eq!(editor.cursor(), (0, 0));
    }

    #[test]
    fn backspace_at_cursor() {
        let mut editor = Editor::new(10);

        editor.add('a');
        editor.add('b');
        editor.move_left();
        editor.backspace();

        assert_eq!(editor.render(), "b");
        assert_eq!(editor.cursor(), (0, 0));
    }

    #[test]
    fn backspace_at_start_of_buffer() {
        let mut editor = Editor::new(10);
        editor.backspace();
    }

    #[test]
    fn backspace_at_start_of_line() {
        let mut editor = Editor::new(1);

        editor.add('a');
        editor.add('b');
        editor.move_left();
        editor.backspace();

        assert_eq!(editor.render(), "b");
        assert_eq!(editor.cursor(), (0, 0));
    }

    #[test]
    fn backspace_at_start_of_para() {
        let mut editor = Editor::new(1);

        editor.add('a');
        editor.add('b');
        editor.enter();
        editor.add('c');
        editor.move_left();
        editor.backspace();

        assert_eq!(editor.render(), "a\nb\nc");
        assert_eq!(editor.cursor(), (1, 1));
    }

    #[test]
    fn move_cursor_left() {
        let mut editor = Editor::new(10);

        editor.add('a');
        editor.move_left();

        assert_eq!(editor.cursor(), (0, 0));
    }

    #[test]
    fn move_cursor_right() {
        let mut editor = Editor::new(10);

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
        let mut editor = Editor::new(10);

        editor.enter();
        editor.move_up();

        assert_eq!(editor.cursor(), (0, 0));
    }

    #[test]
    fn move_cursor_down() {
        let mut editor = Editor::new(10);

        editor.enter();
        editor.enter();
        editor.move_up();
        editor.move_up();
        editor.move_down();

        assert_eq!(editor.cursor(), (2, 0));
    }

    #[test]
    fn cursor_movement_edge_cases() {
        let mut editor = Editor::new(10);

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
        let mut editor = Editor::new(4);

        for c in "foo bar".chars() {
            editor.add(c);
        }
        assert_eq!(editor.render(), "foo \nbar");

        for _ in 0..4 {
            editor.move_left();
        }

        assert_eq!(editor.cursor(), (0, 4));
    }

    #[test]
    fn wrap_cursor_around_at_start_of_para() {
        let mut editor = Editor::new(10);

        editor.add('a');
        editor.enter();
        editor.move_left();

        assert_eq!(editor.cursor(), (0, 1));
    }

    #[test]
    fn wrap_cursor_around_at_end_of_line() {
        let mut editor = Editor::new(4);

        for c in "foo bar".chars() {
            editor.add(c);
        }
        assert_eq!(editor.render(), "foo \nbar");

        editor.move_up();
        editor.move_right();
        editor.move_right();

        assert_eq!(editor.cursor(), (1, 0));
    }

    #[test]
    fn wrap_cursor_around_at_end_of_para() {
        let mut editor = Editor::new(10);

        editor.add('a');
        editor.enter();
        editor.add('b');
        editor.move_up();
        editor.move_right();

        assert_eq!(editor.cursor(), (2, 0));
    }

    #[test]
    fn move_to_start_of_line_if_at_top_when_moving_up() {
        let mut editor = Editor::new(10);

        editor.add('a');
        editor.add('b');
        editor.move_up();

        assert_eq!(editor.cursor(), (0, 0));
    }

    #[test]
    fn move_to_end_of_line_if_at_bottom_when_moving_down() {
        let mut editor = Editor::new(10);

        editor.add('a');
        editor.add('b');
        editor.move_left();
        editor.move_down();

        assert_eq!(editor.cursor(), (0, 2));
    }

    #[test]
    fn clamp_cursor_to_line_when_moving_up_and_down() {
        let mut editor = Editor::new(4);

        for c in "abc d efg".chars() {
            editor.add(c);
        }
        assert_eq!(editor.render(), "abc \nd \nefg");
        assert_eq!(editor.cursor(), (2, 3));

        editor.move_up();
        assert_eq!(editor.cursor(), (1, 2));

        editor.move_up();
        editor.move_right();
        editor.move_down();
        assert_eq!(editor.cursor(), (1, 2));
    }

    #[test]
    fn enter_at_start_of_line() {
        let mut editor = Editor::new(10);

        editor.add('a');
        editor.move_left();
        editor.enter();
        editor.add('b');

        assert_eq!(editor.render(), "\n\nba");
        assert_eq!(editor.cursor(), (2, 1));
    }

    #[test]
    fn enter_at_end_of_line() {
        let mut editor = Editor::new(10);

        editor.add('a');
        editor.enter();
        editor.add('b');

        assert_eq!(editor.render(), "a\n\nb");
        assert_eq!(editor.cursor(), (2, 1));
    }

    #[test]
    fn enter_in_middle_of_line() {
        let mut editor = Editor::new(10);

        editor.add('a');
        editor.add('b');
        editor.move_left();
        editor.enter();

        assert_eq!(editor.render(), "a\n\nb");
        assert_eq!(editor.cursor(), (2, 0));
    }

    #[test]
    fn wrap_text_if_over_width_limit() {
        let mut editor = Editor::new(8);

        for c in "foo bar baz".chars() {
            editor.add(c);
        }

        assert_eq!(editor.render(), "foo bar \nbaz");
    }

    #[test]
    fn resize() {
        let mut editor = Editor::new(4);

        for c in "foo bar".chars() {
            editor.add(c);
        }
        assert_eq!(editor.render(), "foo \nbar");

        editor.resize(7);
        assert_eq!(editor.render(), "foo bar");
    }

    #[test]
    fn rewrap_when_adding_text() {
        let mut editor = Editor::new(2);

        editor.add('a');
        editor.add('b');
        assert_eq!(editor.cursor(), (0, 2));

        editor.add('c');
        assert_eq!(editor.cursor(), (1, 1));
    }

    #[test]
    fn rewrap_when_backspacing() {
        let mut editor = Editor::new(5);

        for c in "foo bar".chars() {
            editor.add(c);
        }
        assert_eq!(editor.render(), "foo \nbar");

        editor.backspace();
        editor.backspace();
        assert_eq!(editor.render(), "foo b");
    }

    #[test]
    fn rewrap_when_splitting_paragraphs() {
        let mut editor = Editor::new(8);

        for c in "foo bar baz quux".chars() {
            editor.add(c);
        }
        assert_eq!(editor.render(), "foo bar \nbaz quux");

        for _ in 0..13 {
            editor.move_left();
        }
        editor.enter();

        assert_eq!(editor.render(), "foo \n\nbar baz \nquux");
    }
}
