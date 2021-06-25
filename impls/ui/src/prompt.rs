use crate::types::StyledText;
use crate::{TextField, WrappedLabel};

#[derive(Debug)]
pub struct Prompt {
    label: WrappedLabel,
    text_field: TextField,
}

impl Prompt {
    pub fn new(prompt: &'static str, width: usize) -> Self {
        Self {
            label: WrappedLabel::new(prompt, width),
            text_field: TextField::new(width),
        }
    }

    pub fn render(&self) -> Vec<StyledText<'_>> {
        self.label
            .render()
            .into_iter()
            .map(StyledText::Bold)
            .chain(std::iter::once(StyledText::Regular(
                self.text_field.render(),
            )))
            .collect()
    }

    pub fn contents(&self) -> &str {
        self.text_field.contents()
    }

    pub fn cursor(&self) -> (usize, usize) {
        (self.label.num_rows(), self.text_field.cursor())
    }

    pub fn resize(&mut self, width: usize) {
        self.label.resize(width);
        self.text_field.resize(width);
    }

    pub fn add(&mut self, s: &str) {
        self.text_field.add(s);
    }

    pub fn backspace(&mut self) {
        self.text_field.backspace();
    }

    pub fn move_left(&mut self) {
        self.text_field.move_left();
    }

    pub fn move_right(&mut self) {
        self.text_field.move_right();
    }

    pub fn move_up(&mut self) {
        self.text_field.move_up();
    }

    pub fn move_down(&mut self) {
        self.text_field.move_down();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use StyledText::*;

    #[test]
    fn wrap_at_given_width() {
        let prompt = Prompt::new("Enter a nickname", 8);

        assert_eq!(
            prompt.render(),
            [Bold("Enter a "), Bold("nickname"), Regular("")]
        );
        assert_eq!(prompt.cursor(), (2, 0));
    }

    #[test]
    fn add_text() {
        let mut prompt = Prompt::new("Enter a nickname", 8);

        prompt.add("john");

        assert_eq!(
            prompt.render(),
            [Bold("Enter a "), Bold("nickname"), Regular("john")]
        );
        assert_eq!(prompt.cursor(), (2, 4));
    }

    #[test]
    fn backspace() {
        let mut prompt = Prompt::new("Enter a file to upload", 15);

        prompt.add("fooo");
        prompt.backspace();

        assert_eq!(
            prompt.render(),
            [Bold("Enter a file "), Bold("to upload"), Regular("foo")]
        );
        assert_eq!(prompt.cursor(), (2, 3));
    }

    #[test]
    fn cursor_movement() {
        let mut prompt = Prompt::new("Choose a color", 10);

        prompt.add("purple");

        prompt.move_left();
        assert_eq!(prompt.cursor(), (2, 5));

        prompt.move_right();
        assert_eq!(prompt.cursor(), (2, 6));

        prompt.move_up();
        assert_eq!(prompt.cursor(), (2, 0));

        prompt.move_down();
        assert_eq!(prompt.cursor(), (2, 6));
    }

    #[test]
    fn resize() {
        let mut prompt = Prompt::new("This is a test", 10);

        prompt.add("foo bar baz quux");

        assert_eq!(
            prompt.render(),
            [Bold("This is a "), Bold("test"), Regular("r baz quux")]
        );
        assert_eq!(prompt.cursor(), (2, 10));

        prompt.resize(20);
        assert_eq!(
            prompt.render(),
            [Bold("This is a test"), Regular("foo bar baz quux")]
        );
        assert_eq!(prompt.cursor(), (1, 16));
    }

    #[test]
    fn contents() {
        let mut prompt = Prompt::new("Enter feedback", 10);

        prompt.add("test");

        assert_eq!(prompt.contents(), "test");
    }
}
