use text::Text;
use unicode_width::UnicodeWidthStr;

pub(super) fn split_into_words(text: Text<'_>, width: usize) -> impl Iterator<Item = Text<'_>> {
    WordSplitter {
        text,
        grapheme_pos: 0,
        visual_pos: 0,
        width,
    }
}

#[derive(Debug)]
struct WordSplitter<'a> {
    text: Text<'a>,
    grapheme_pos: usize,
    visual_pos: usize,
    width: usize,
}

impl<'a> Iterator for WordSplitter<'a> {
    type Item = Text<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.at_end() {
            return None;
        }

        let next_word_boundary = self
            .current_text()
            .find(" ")
            .map_or_else(|| self.current_text().len(), |space_idx| space_idx + 1);

        let word = if next_word_boundary > self.width {
            self.chunk()
        } else {
            self.current_text().slice(..next_word_boundary)
        };

        self.grapheme_pos += word.len();
        self.visual_pos += word.width();

        Some(word)
    }
}

impl<'a> WordSplitter<'a> {
    /// Returns text with a width between 1 and `self.width`.
    fn chunk(&self) -> Text<'a> {
        let mut num_graphemes = 1;
        let mut text;

        loop {
            // step forward one grapheme
            num_graphemes += 1;
            text = self.current_text().slice(..num_graphemes);

            let is_too_wide = text.width() > self.width;
            if is_too_wide {
                // step back one grapheme
                num_graphemes -= 1;
                text = self.current_text().slice(..num_graphemes);

                break;
            }

            let none_left = num_graphemes == self.text.len();
            if none_left {
                break;
            }
        }

        assert!(text.width() > 0);
        assert!(text.width() <= self.width);

        text
    }

    fn current_text(&self) -> Text<'a> {
        self.text.slice(self.grapheme_pos..)
    }

    fn at_end(&self) -> bool {
        self.visual_pos == self.text.width()
    }
}
