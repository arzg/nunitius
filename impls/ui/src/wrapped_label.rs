use text::{Text, TextBuf};

#[derive(Debug)]
pub struct WrappedLabel {
    text: Text<'static>,
    wrapped: Vec<TextBuf>,
    width: usize,
}

impl WrappedLabel {
    pub fn new(text: &'static str, width: usize) -> Self {
        let text = Text::new(text);
        let wrapped = text::wrap(&text, width);

        Self {
            text,
            wrapped,
            width,
        }
    }

    pub fn render(&self) -> Vec<&str> {
        self.wrapped.iter().map(TextBuf::as_str).collect()
    }

    pub fn num_rows(&self) -> usize {
        self.wrapped.len()
    }

    pub fn resize(&mut self, width: usize) {
        self.width = width;
        self.wrapped = text::wrap(&self.text, self.width);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let label = WrappedLabel::new("foo bar baz", 8);

        assert_eq!(label.render(), ["foo bar ", "baz"]);
        assert_eq!(label.num_rows(), 2);
    }

    #[test]
    fn resize() {
        let mut label = WrappedLabel::new("foo bar baz", 8);

        assert_eq!(label.render(), ["foo bar ", "baz"]);
        assert_eq!(label.num_rows(), 2);

        label.resize(4);

        assert_eq!(label.render(), ["foo ", "bar ", "baz"]);
        assert_eq!(label.num_rows(), 3);
    }
}
