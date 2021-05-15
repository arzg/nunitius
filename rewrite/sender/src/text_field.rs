use text::{Text, TextBuf};
use unicode_width::UnicodeWidthStr;

#[derive(Debug)]
pub(crate) struct TextField {
    buffer: TextBuf,
    cursor: usize,
    amount_scrolled: usize,
    width: usize,
}

impl TextField {
    pub(crate) fn new(width: usize) -> Self {
        Self {
            buffer: TextBuf::default(),
            cursor: 0,
            amount_scrolled: 0,
            width,
        }
    }

    pub(crate) fn render(&self) -> &str {
        if self.can_all_text_fit_on_screen() {
            return self.buffer.as_str();
        }

        self.buffer
            .slice(self.amount_scrolled..self.amount_scrolled + self.width)
            .as_str()
    }

    pub(crate) fn contents(&self) -> &str {
        self.buffer.as_str()
    }

    pub(crate) fn cursor(&self) -> usize {
        let text_before_cursor = self.buffer.slice(..self.cursor - self.amount_scrolled);
        text_before_cursor.width()
    }

    pub(crate) fn resize(&mut self, width: usize) {
        self.width = width;
        self.adjust_scroll();
    }

    pub(crate) fn add(&mut self, s: &str) {
        self.buffer.insert(self.cursor, s);

        let text = Text::new(s);
        self.cursor += text.len();

        self.adjust_scroll();
    }

    pub(crate) fn backspace(&mut self) {
        if self.at_start() {
            return;
        }

        self.buffer.remove(self.cursor - 1);
        self.cursor -= 1;

        self.adjust_scroll();
    }

    pub(crate) fn move_left(&mut self) {
        if self.at_start() {
            return;
        }

        self.cursor -= 1;
        self.adjust_scroll();
    }

    pub(crate) fn move_right(&mut self) {
        if self.at_end() {
            return;
        }

        self.cursor += 1;
        self.adjust_scroll();
    }

    pub(crate) fn move_up(&mut self) {
        self.cursor = 0;
        self.adjust_scroll();
    }

    pub(crate) fn move_down(&mut self) {
        self.cursor = self.buffer.len();
        self.adjust_scroll();
    }

    fn adjust_scroll(&mut self) {
        if self.can_all_text_fit_on_screen() {
            self.amount_scrolled = 0;
        } else if self.cursor_before_left() {
            self.scroll_cursor_to_left();
        } else if self.cursor_after_right() {
            self.scroll_cursor_to_right();
        } else if self.scrolled_past_right() {
            self.scroll_buffer_to_right();
        }
    }

    fn scroll_cursor_to_left(&mut self) {
        self.amount_scrolled = self.cursor;
    }

    fn scroll_cursor_to_right(&mut self) {
        self.amount_scrolled = self.cursor - self.width;
    }

    fn scroll_buffer_to_right(&mut self) {
        self.amount_scrolled = self.buffer.len() - self.width;
    }

    fn cursor_before_left(&self) -> bool {
        self.cursor < self.amount_scrolled
    }

    fn cursor_after_right(&self) -> bool {
        self.cursor > self.amount_scrolled + self.width
    }

    fn scrolled_past_right(&self) -> bool {
        self.amount_scrolled > self.buffer.len() - self.width
    }

    fn can_all_text_fit_on_screen(&self) -> bool {
        self.buffer.len() <= self.width
    }

    fn at_start(&self) -> bool {
        self.cursor == 0
    }

    fn at_end(&self) -> bool {
        self.cursor == self.buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_text() {
        let mut text_field = TextField::new(10);

        text_field.add("a");

        assert_eq!(text_field.render(), "a");
        assert_eq!(text_field.cursor(), 1);
    }

    #[test]
    fn add_text_at_cursor() {
        let mut text_field = TextField::new(10);

        text_field.add("ac");
        text_field.move_left();
        text_field.add("b");

        assert_eq!(text_field.render(), "abc");
        assert_eq!(text_field.cursor(), 2);
    }

    #[test]
    fn backspace() {
        let mut text_field = TextField::new(10);

        text_field.add("a");
        text_field.backspace();

        assert_eq!(text_field.render(), "");
        assert_eq!(text_field.cursor(), 0);
    }

    #[test]
    fn backspace_at_cursor() {
        let mut text_field = TextField::new(10);

        text_field.add("ab");
        text_field.move_left();
        text_field.backspace();

        assert_eq!(text_field.render(), "b");
        assert_eq!(text_field.cursor(), 0);
    }

    #[test]
    fn backspace_at_start() {
        let mut text_field = TextField::new(10);

        text_field.backspace();

        assert_eq!(text_field.render(), "");
        assert_eq!(text_field.cursor(), 0);
    }

    #[test]
    fn move_cursor_left() {
        let mut text_field = TextField::new(10);

        text_field.add("a");
        text_field.move_left();

        assert_eq!(text_field.cursor(), 0);
    }

    #[test]
    fn move_cursor_right() {
        let mut text_field = TextField::new(10);

        text_field.add("abc");
        text_field.move_left();
        text_field.move_left();
        text_field.move_right();

        assert_eq!(text_field.cursor(), 2);
    }

    #[test]
    fn move_to_start_when_moving_up() {
        let mut text_field = TextField::new(10);

        text_field.add("abc");
        text_field.move_up();

        assert_eq!(text_field.cursor(), 0);
    }

    #[test]
    fn move_to_end_when_moving_down() {
        let mut text_field = TextField::new(10);

        text_field.add("abc");
        text_field.move_left();
        text_field.move_left();
        text_field.move_down();

        assert_eq!(text_field.cursor(), 3);
    }

    #[test]
    fn cursor_movement_edge_cases() {
        let mut text_field = TextField::new(10);

        text_field.move_left();
        assert_eq!(text_field.cursor(), 0);

        text_field.move_right();
        assert_eq!(text_field.cursor(), 0);

        text_field.move_up();
        assert_eq!(text_field.cursor(), 0);

        text_field.move_down();
        assert_eq!(text_field.cursor(), 0);
    }

    #[test]
    fn move_cursor_left_at_start() {
        let mut text_field = TextField::new(10);

        text_field.add("a");
        text_field.move_left();
        text_field.move_left();

        assert_eq!(text_field.cursor(), 0);
    }

    #[test]
    fn move_cursor_right_at_end() {
        let mut text_field = TextField::new(10);

        text_field.add("a");
        text_field.move_right();

        assert_eq!(text_field.cursor(), 1);
    }

    #[test]
    fn scroll_right_when_adding_text_past_right() {
        let mut text_field = TextField::new(3);

        text_field.add("abc");
        assert_eq!(text_field.render(), "abc");
        assert_eq!(text_field.cursor(), 3);

        text_field.add("d");
        assert_eq!(text_field.render(), "bcd");
        assert_eq!(text_field.cursor(), 3);
    }

    #[test]
    fn scroll_left_when_backspacing_to_fill_whole_screen() {
        let mut text_field = TextField::new(4);

        text_field.add("foo bar");
        assert_eq!(text_field.render(), " bar");
        assert_eq!(text_field.cursor(), 4);

        text_field.move_left();
        text_field.backspace();
        assert_eq!(text_field.render(), "o br");
        assert_eq!(text_field.cursor(), 3);
    }

    #[test]
    fn scroll_left_when_moving_past_left() {
        let mut text_field = TextField::new(3);

        text_field.add("abcd");
        assert_eq!(text_field.render(), "bcd");
        assert_eq!(text_field.cursor(), 3);

        for _ in 0..4 {
            text_field.move_left();
        }
        assert_eq!(text_field.render(), "abc");
        assert_eq!(text_field.cursor(), 0);
    }

    #[test]
    fn scroll_right_when_moving_past_left() {
        let mut text_field = TextField::new(3);

        text_field.add("abcd");
        text_field.move_up();
        assert_eq!(text_field.render(), "abc");
        assert_eq!(text_field.cursor(), 0);

        for _ in 0..4 {
            text_field.move_right();
        }
        assert_eq!(text_field.render(), "bcd");
        assert_eq!(text_field.cursor(), 3);
    }

    #[test]
    fn scroll_left_when_moving_up_if_needed() {
        let mut text_field = TextField::new(3);

        text_field.add("abcd");
        assert_eq!(text_field.render(), "bcd");
        assert_eq!(text_field.cursor(), 3);

        text_field.move_up();
        assert_eq!(text_field.render(), "abc");
        assert_eq!(text_field.cursor(), 0);
    }

    #[test]
    fn scroll_right_when_moving_down_if_needed() {
        let mut text_field = TextField::new(3);

        text_field.add("abcd");
        text_field.move_up();
        assert_eq!(text_field.render(), "abc");
        assert_eq!(text_field.cursor(), 0);

        text_field.move_down();
        assert_eq!(text_field.render(), "bcd");
        assert_eq!(text_field.cursor(), 3);
    }

    #[test]
    fn resize() {
        let mut text_field = TextField::new(2);

        text_field.add("abc");
        assert_eq!(text_field.render(), "bc");
        assert_eq!(text_field.cursor(), 2);

        text_field.resize(10);
        assert_eq!(text_field.render(), "abc");
        assert_eq!(text_field.cursor(), 3);
    }

    #[test]
    fn add_wide_grapheme() {
        let mut text_field = TextField::new(10);

        text_field.add("🦀");

        assert_eq!(text_field.render(), "🦀");
        assert_eq!(text_field.cursor(), 2);
    }

    #[test]
    fn backspace_grapheme() {
        let mut text_field = TextField::new(10);

        text_field.add("🧑🏾‍🌾");
        text_field.backspace();

        assert_eq!(text_field.render(), "");
        assert_eq!(text_field.cursor(), 0);
    }

    #[test]
    fn scroll_at_grapheme_boundaries() {
        let mut text_field = TextField::new(2);

        text_field.add("🙄😦😑");

        assert_eq!(text_field.render(), "😦😑");
        assert_eq!(text_field.cursor(), 4);
    }

    #[test]
    fn contents() {
        let mut text_field = TextField::new(10);

        text_field.add("foo bar");

        assert_eq!(text_field.contents(), "foo bar");
    }
}
