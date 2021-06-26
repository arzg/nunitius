use delegate::delegate;
use ui::types::StyledText;
use ui::Prompt;
use ui::WrappedLabel;

#[derive(Debug)]
pub struct LoggingInView {
    nickname_prompt: Prompt,
    taken: Option<WrappedLabel>,
    width: usize,
}

impl LoggingInView {
    pub fn new(width: usize) -> Self {
        Self {
            nickname_prompt: Prompt::new("Enter a nickname", width),
            taken: None,
            width,
        }
    }

    pub fn render(&self) -> Vec<StyledText<'_>> {
        let mut output = self.nickname_prompt.render();

        if let Some(ref label) = self.taken {
            output.extend(label.render().into_iter().map(StyledText::Red));
        }

        output
    }

    pub fn resize(&mut self, width: usize) {
        self.nickname_prompt.resize(width);

        if let Some(ref mut label) = self.taken {
            label.resize(width);
        }
    }

    pub fn mark_nickname_taken(&mut self) {
        self.taken = Some(WrappedLabel::new("nickname is taken", self.width));
    }

    pub fn add(&mut self, s: &str) {
        self.nickname_prompt.add(s);
        self.taken = None;
    }

    pub fn backspace(&mut self) {
        self.nickname_prompt.backspace();
        self.taken = None;
    }

    delegate! {
        to self.nickname_prompt {
            pub fn contents(&self) -> &str;
            pub fn cursor(&self) -> (usize, usize);
            pub fn move_left(&mut self);
            pub fn move_right(&mut self);
            pub fn move_up(&mut self);
            pub fn move_down(&mut self);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use StyledText::*;

    #[test]
    fn empty() {
        let view = LoggingInView::new(20);

        assert_eq!(view.render(), [Bold("Enter a nickname"), Regular("")]);
        assert_eq!(view.cursor(), (1, 0));
    }

    #[test]
    fn add_nickname() {
        let mut view = LoggingInView::new(20);

        view.add("me");

        assert_eq!(view.render(), [Bold("Enter a nickname"), Regular("me")]);
        assert_eq!(view.cursor(), (1, 2));
    }

    #[test]
    fn taken() {
        let mut view = LoggingInView::new(20);

        view.add("foo");
        view.mark_nickname_taken();

        assert_eq!(
            view.render(),
            [
                Bold("Enter a nickname"),
                Regular("foo"),
                Red("nickname is taken")
            ]
        );
        assert_eq!(view.cursor(), (1, 3));
    }

    #[test]
    fn wrap_taken() {
        let mut view = LoggingInView::new(10);

        view.add("me");
        view.mark_nickname_taken();

        assert_eq!(
            view.render(),
            [
                Bold("Enter a "),
                Bold("nickname"),
                Regular("me"),
                Red("nickname "),
                Red("is taken")
            ]
        );
        assert_eq!(view.cursor(), (2, 2));
    }

    #[test]
    fn untaken_after_edit() {
        let mut view = LoggingInView::new(20);

        view.add("ferris");
        view.mark_nickname_taken();
        view.add("a");

        assert_eq!(
            view.render(),
            [Bold("Enter a nickname"), Regular("ferrisa"),]
        );
        assert_eq!(view.cursor(), (1, 7));

        view.mark_nickname_taken();
        view.backspace();

        assert_eq!(
            view.render(),
            [Bold("Enter a nickname"), Regular("ferris"),]
        );
        assert_eq!(view.cursor(), (1, 6));
    }

    #[test]
    fn contents() {
        let mut view = LoggingInView::new(20);

        view.add("foo");
        view.backspace();
        view.backspace();
        view.backspace();
        view.add("bar");

        assert_eq!(view.contents(), "bar");
    }
}
