mod wrap;
use wrap::wrap;

use std::ops::Range;

#[derive(Debug)]
pub(crate) struct Editor {
    buffer: Vec<String>,
    line: usize,
    column: usize,
    width: usize,
}

impl Editor {
    pub(crate) fn new(width: usize) -> Self {
        Self {
            buffer: vec![String::new()],
            line: 0,
            column: 0,
            width,
        }
    }

    pub(crate) fn render(&self) -> String {
        self.buffer.join("\n")
    }

    pub(crate) fn resize(&mut self, width: usize) {
        self.width = width;
        self.rewrap();
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
        self.rewrap_current_para();
    }

    pub(crate) fn backspace(&mut self) {
        if self.at_start_of_buffer() {
            return;
        }

        if self.at_start_of_line() {
            self.join_lines();
            return;
        }

        self.buffer[self.line].remove(self.column - 1);
        self.column -= 1;
        self.rewrap_current_para();
    }

    fn join_lines(&mut self) {
        self.line -= 1;
        self.move_to_end_of_line();

        let line = self.buffer.remove(self.line + 1);
        self.buffer[self.line].push_str(&line);

        if !self.current_para_idx().is_empty() {
            self.rewrap_current_para();
        }
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

    fn rewrap(&mut self) {
        let cursor_idx = self.cursor_idx();
        let para_idxs = self.para_idxs();

        for para_idx in para_idxs {
            let para = &self.buffer[para_idx.clone()];
            let wrapped = wrap(para.iter().map(|s| s.as_str()), self.width);

            // remove existing non-rewrapped paragraph
            drop(self.buffer.drain(para_idx.clone()));

            for (idx, line) in wrapped.into_iter().enumerate() {
                self.buffer.insert(para_idx.start + idx, line);
            }
        }

        self.move_to(cursor_idx);
    }

    fn rewrap_current_para(&mut self) {
        let current_para_idx = self.current_para_idx();

        let current_para = self.buffer[current_para_idx.clone()]
            .iter()
            .map(|s| s.as_str());

        let wrapped = wrap(current_para, self.width);

        let cursor_idx = self.cursor_idx();

        // remove current para
        drop(self.buffer.drain(current_para_idx.clone()));

        for (idx, line) in wrapped.into_iter().enumerate() {
            self.buffer.insert(current_para_idx.start + idx, line);
        }

        self.move_to(cursor_idx);
    }

    fn para_idxs(&self) -> Vec<Range<usize>> {
        let para_separators: Vec<_> = self
            .buffer
            .iter()
            .enumerate()
            .filter_map(|(idx, line)| Self::is_line_para_separator(line).then(|| idx))
            .collect();

        let mut para_idxs: Vec<_> = para_separators
            .windows(2)
            .map(|separators| {
                let start = separators[0] + 1;
                let end = separators[1];

                start..end
            })
            .collect();

        if para_separators.is_empty() {
            para_idxs.push(0..self.buffer.len());
        } else {
            if para_separators[0] != 0 {
                para_idxs.insert(0, 0..para_separators[0]);
            }

            if *para_separators.last().unwrap() != self.buffer.len() {
                para_idxs.push(para_separators.last().unwrap() + 1..self.buffer.len());
            }
        }

        para_idxs
    }

    fn current_para_idx(&self) -> Range<usize> {
        let on_paragraph_separator = Self::is_line_para_separator(&self.buffer[self.line]);
        if on_paragraph_separator {
            return self.line..self.line + 1;
        }

        // both of these include the current line
        let lines_above_cursor = self.buffer.iter().enumerate().take(self.line + 1);
        let mut lines_below_cursor = self.buffer.iter().enumerate().skip(self.line);

        // the output doesn’t include paragraph separators

        let start_line_idx = lines_above_cursor
            .rev()
            .find_map(|(idx, line)| Self::is_line_para_separator(line).then(|| idx + 1))
            .unwrap_or(0);

        let end_line_idx = lines_below_cursor
            .find_map(|(idx, line)| Self::is_line_para_separator(line).then(|| idx))
            .unwrap_or(self.buffer.len());

        start_line_idx..end_line_idx
    }

    fn is_line_para_separator(line: &str) -> bool {
        line.is_empty() || line.chars().all(char::is_whitespace)
    }

    fn move_to(&mut self, idx: usize) {
        self.line = 0;
        self.column = 0;
        let mut elems_stepped = 0;

        'outer: for line in &self.buffer {
            if elems_stepped == idx {
                break 'outer;
            }

            if line.is_empty() {
                elems_stepped += 1;
            } else {
                for _ in line.as_bytes() {
                    self.column += 1;
                    elems_stepped += 1;

                    if elems_stepped == idx {
                        break 'outer;
                    }
                }
            }

            self.line += 1;
            self.column = 0;
        }
    }

    fn move_to_end_of_line(&mut self) {
        self.column = self.buffer[self.line].len();
    }

    fn at_start_of_buffer(&self) -> bool {
        self.at_first_line() && self.at_start_of_line()
    }

    fn cursor_idx(&self) -> usize {
        let num_bytes_before_cursor = self.buffer[..self.line]
            .iter()
            .map(String::len)
            .sum::<usize>()
            + self.column;

        let num_empty_lines_before_cursor = self.buffer[..self.line]
            .iter()
            .filter(|line| line.is_empty())
            .count();

        num_bytes_before_cursor + num_empty_lines_before_cursor
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
        self.line == self.buffer.len() - 1 || self.buffer.len() == 1
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
        let mut editor = Editor::new(10);

        editor.add('a');
        editor.enter();
        editor.enter();
        editor.add('b');
        editor.move_left();
        editor.backspace();

        assert_eq!(editor.render(), "ab");
        assert_eq!(editor.cursor(), (0, 1));
    }

    #[test]
    fn backspace_at_start_of_line_with_empty_line_above() {
        let mut editor = Editor::new(10);

        editor.add('a');
        editor.enter();
        editor.enter();
        editor.enter();
        editor.add('b');
        editor.move_left();
        editor.backspace();

        assert_eq!(editor.render(), "a\n\nb");
        assert_eq!(editor.cursor(), (2, 0));
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

        assert_eq!(editor.cursor(), (1, 0));
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
        let mut editor = Editor::new(10);

        editor.add('a');
        editor.enter();
        editor.move_left();

        assert_eq!(editor.cursor(), (0, 1));
    }

    #[test]
    fn wrap_cursor_around_at_end_of_line() {
        let mut editor = Editor::new(10);

        editor.add('a');
        editor.enter();
        editor.enter();
        editor.add('b');
        editor.move_up();
        editor.move_up();
        editor.move_right();

        assert_eq!(editor.cursor(), (1, 0));
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
    fn enter_at_start_of_line() {
        let mut editor = Editor::new(10);

        editor.add('a');
        editor.move_left();
        editor.enter();
        editor.add('b');

        assert_eq!(editor.render(), "\nba");
        assert_eq!(editor.cursor(), (1, 1));
    }

    #[test]
    fn enter_at_end_of_line() {
        let mut editor = Editor::new(10);

        editor.add('a');
        editor.enter();
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

        assert_eq!(editor.render(), "a\nb");
        assert_eq!(editor.cursor(), (1, 0));
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
}
