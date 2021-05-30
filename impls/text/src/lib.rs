use std::borrow::Cow;
use std::ops::{Bound, RangeBounds};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Text<'a> {
    s: &'a str,
    grapheme_idxs: Cow<'a, [usize]>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TextBuf {
    s: String,
    grapheme_idxs: Vec<usize>,
}

impl<'a> Text<'a> {
    pub fn new(s: &'a str) -> Self {
        let grapheme_idxs = calculate_grapheme_idxs(s);

        Self {
            s,
            grapheme_idxs: Cow::Owned(grapheme_idxs),
        }
    }

    pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
        let start_idx = match range.start_bound() {
            Bound::Included(&idx) => self.grapheme_idxs[idx],
            Bound::Excluded(&idx) => self.grapheme_idxs[idx + 1],
            Bound::Unbounded => 0,
        };

        let end_idx = match range.end_bound() {
            Bound::Included(&idx) => {
                if idx + 1 == self.grapheme_idxs.len() {
                    self.s.len()
                } else {
                    self.grapheme_idxs[idx + 1]
                }
            }

            Bound::Excluded(&idx) => {
                if idx == self.grapheme_idxs.len() {
                    self.s.len()
                } else {
                    self.grapheme_idxs[idx]
                }
            }

            Bound::Unbounded => self.s.len(),
        };

        Self::new(&self.s[start_idx..end_idx])
    }

    pub fn split(&self, idx: usize) -> (Self, Self) {
        let (before, after) = self.s.split_at(idx);
        (Self::new(before), Self::new(after))
    }

    pub fn find(&self, s: &str) -> Option<usize> {
        let wanted_byte_idx = self.s.find(s)?;

        self.grapheme_idxs
            .iter()
            .enumerate()
            .find_map(|(grapheme_idx, &byte_idx)| {
                (byte_idx == wanted_byte_idx).then(|| grapheme_idx)
            })
    }

    pub fn len(&self) -> usize {
        self.grapheme_idxs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.s.is_empty()
    }

    pub fn into_text_buf(self) -> TextBuf {
        TextBuf {
            s: self.s.to_string(),
            grapheme_idxs: self.grapheme_idxs.into(),
        }
    }

    pub fn as_str(&self) -> &'a str {
        self.s
    }
}

impl TextBuf {
    pub fn new(s: String) -> Self {
        let grapheme_idxs = calculate_grapheme_idxs(&s);
        Self { s, grapheme_idxs }
    }

    pub fn slice(&self, range: impl RangeBounds<usize>) -> Text<'_> {
        self.as_text().slice(range)
    }

    pub fn push(&mut self, s: &str) {
        self.s.push_str(s);
        self.recalculate_grapheme_idxs();
    }

    pub fn remove(&mut self, idx: usize) -> Self {
        let grapheme_start = self.grapheme_idxs[idx];

        let grapheme_end = if idx == self.grapheme_idxs.len() - 1 {
            self.s.len()
        } else {
            self.grapheme_idxs[idx + 1]
        };

        let removed_text = Self::new(self.s.drain(grapheme_start..grapheme_end).collect());
        self.recalculate_grapheme_idxs();

        removed_text
    }

    pub fn insert(&mut self, idx: usize, s: &str) {
        let idx = if idx == 0 {
            0
        } else if idx == self.grapheme_idxs.len() {
            self.s.len()
        } else {
            self.grapheme_idxs[idx]
        };

        self.s.insert_str(idx, s);
        self.recalculate_grapheme_idxs();
    }

    pub fn split(&self, idx: usize) -> (Text<'_>, Text<'_>) {
        self.as_text().split(idx)
    }

    pub fn find(&self, s: &str) -> Option<usize> {
        self.as_text().find(s)
    }

    pub fn len(&self) -> usize {
        self.grapheme_idxs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.s.is_empty()
    }

    pub fn as_text(&self) -> Text<'_> {
        Text {
            s: &self.s,
            grapheme_idxs: Cow::Borrowed(&self.grapheme_idxs),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.s
    }

    fn recalculate_grapheme_idxs(&mut self) {
        self.grapheme_idxs = calculate_grapheme_idxs(&self.s);
    }
}

impl UnicodeWidthStr for Text<'_> {
    fn width(&self) -> usize {
        self.s.width()
    }

    fn width_cjk(&self) -> usize {
        self.s.width_cjk()
    }
}

impl UnicodeWidthStr for TextBuf {
    fn width(&self) -> usize {
        self.s.width()
    }

    fn width_cjk(&self) -> usize {
        self.s.width_cjk()
    }
}

fn calculate_grapheme_idxs(s: &str) -> Vec<usize> {
    s.grapheme_indices(true).map(|(idx, _)| idx).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice() {
        let text = Text::new("å🧓🏻bc");
        assert_eq!(text.slice(1..3), Text::new("🧓🏻b"));

        let text = TextBuf::new("å🧓🏻bc".to_string());
        assert_eq!(text.slice(1..3), Text::new("🧓🏻b"));
    }

    #[test]
    fn slice_from() {
        let text = Text::new("🧑‍💻hello");
        assert_eq!(text.slice(1..), Text::new("hello"));

        let text = TextBuf::new("🧑‍💻hello".to_string());
        assert_eq!(text.slice(1..), Text::new("hello"));
    }

    #[test]
    fn slice_to() {
        let text = Text::new("abc🦀🧑🏽def");
        assert_eq!(text.slice(..5), Text::new("abc🦀🧑🏽"));

        let text = TextBuf::new("abc🦀🧑🏽def".to_string());
        assert_eq!(text.slice(..5), Text::new("abc🦀🧑🏽"));
    }

    #[test]
    fn slice_unbounded() {
        let text = Text::new("abc🤷xyz");
        assert_eq!(text.slice(..), Text::new("abc🤷xyz"));

        let text = TextBuf::new("abc🤷xyz".to_string());
        assert_eq!(text.slice(..), Text::new("abc🤷xyz"));
    }

    #[test]
    fn slice_to_end() {
        let text = Text::new("1️⃣2️⃣3️⃣");
        assert_eq!(text.slice(..3), Text::new("1️⃣2️⃣3️⃣"));
        assert_eq!(text.slice(..=2), Text::new("1️⃣2️⃣3️⃣"));

        let text = TextBuf::new("1️⃣2️⃣3️⃣".to_string());
        assert_eq!(text.slice(..3), Text::new("1️⃣2️⃣3️⃣"));
        assert_eq!(text.slice(..=2), Text::new("1️⃣2️⃣3️⃣"));
    }

    #[test]
    fn push() {
        let mut text = TextBuf::new("foo".to_string());
        text.push(" bar");

        assert_eq!(text, TextBuf::new("foo bar".to_string()));
    }

    #[test]
    fn remove() {
        let mut text = TextBuf::new("🙂🙃🙂🙂".to_string());
        assert_eq!(text.remove(1), TextBuf::new("🙃".to_string()));
        assert_eq!(text, TextBuf::new("🙂🙂🙂".to_string()));
    }

    #[test]
    fn remove_at_start() {
        let mut text = TextBuf::new("👍🏽a".to_string());
        assert_eq!(text.remove(0), TextBuf::new("👍🏽".to_string()));
        assert_eq!(text, TextBuf::new("a".to_string()));
    }

    #[test]
    fn remove_at_end() {
        let mut text = TextBuf::new("abc🧑🏻‍🦱".to_string());
        assert_eq!(text.remove(3), TextBuf::new("🧑🏻‍🦱".to_string()));
        assert_eq!(text, TextBuf::new("abc".to_string()));
    }

    #[test]
    fn insert() {
        let mut text = TextBuf::new("🌕🌖🌘🌑".to_string());
        text.insert(2, "🌗");

        assert_eq!(text, TextBuf::new("🌕🌖🌗🌘🌑".to_string()));
    }

    #[test]
    fn insert_at_start() {
        let mut text = TextBuf::default();
        text.insert(0, "a");

        assert_eq!(text, TextBuf::new("a".to_string()));
    }

    #[test]
    fn insert_at_end() {
        let mut text = TextBuf::new("a".to_string());
        text.insert(1, "b");

        assert_eq!(text, TextBuf::new("ab".to_string()));
    }

    #[test]
    fn split() {
        let text = Text::new("a👶🏼b");
        assert_eq!(text.split(1), (Text::new("a"), Text::new("👶🏼b")));

        let text = TextBuf::new("a👶🏼b".to_string());
        assert_eq!(text.split(1), (Text::new("a"), Text::new("👶🏼b")));
    }

    #[test]
    fn find() {
        let text = Text::new("❤️🧡💛💚💙💜");
        assert_eq!(text.find("💚"), Some(3));
        assert_eq!(text.find("a"), None);

        let text = TextBuf::new("❤️🧡💛💚💙💜".to_string());
        assert_eq!(text.find("💚"), Some(3));
        assert_eq!(text.find("a"), None);
    }

    #[test]
    fn len() {
        let text = Text::new("foo🦸🏻bar🤴🏿baz");
        assert_eq!(text.len(), 11);

        let text = TextBuf::new("foo🦸🏻bar🤴🏿baz".to_string());
        assert_eq!(text.len(), 11);
    }

    #[test]
    fn is_empty() {
        let text = Text::default();
        assert!(text.is_empty());

        let text = TextBuf::default();
        assert!(text.is_empty());

        let text = Text::new("");
        assert!(text.is_empty());

        let text = TextBuf::new(String::new());
        assert!(text.is_empty());
    }

    #[test]
    fn as_text() {
        let text = TextBuf::new("Rust".to_string());
        assert_eq!(text.as_text(), Text::new("Rust"));
    }

    #[test]
    fn to_text_buf() {
        let text = Text::new("👁👄👁");
        assert_eq!(text.into_text_buf(), TextBuf::new("👁👄👁".to_string()));
    }

    #[test]
    fn as_str() {
        let text = Text::new("foo");
        assert_eq!(text.as_str(), "foo");

        let text = TextBuf::new("foo".to_string());
        assert_eq!(text.as_str(), "foo");
    }

    #[test]
    fn width() {
        let text = Text::new("😁");
        assert_eq!(UnicodeWidthStr::width(&text), 2);

        let text = TextBuf::new("😁".to_string());
        assert_eq!(UnicodeWidthStr::width(&text), 2);
    }
}
