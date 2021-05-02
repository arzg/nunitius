pub(super) fn wrap<'a>(text: impl Iterator<Item = &'a str>, width: usize) -> Vec<String> {
    let words = text.flat_map(|text| split_into_words(text, width));

    let mut lines = vec![String::new()];
    let mut current_line = 0;

    for word in words {
        if lines[current_line].len() + word.len() > width {
            lines.push(word.to_string());
            current_line += 1;
        } else {
            lines[current_line].push_str(word);
        }
    }

    lines
}

fn split_into_words(text: &str, width: usize) -> impl Iterator<Item = &str> {
    WordSplitter {
        text,
        pos: 0,
        width,
    }
}

struct WordSplitter<'a> {
    text: &'a str,
    pos: usize,
    width: usize,
}

impl<'a> Iterator for WordSplitter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.at_end() {
            return None;
        }

        let next_word_boundary = self
            .current_text()
            .find(' ')
            .map_or_else(|| self.current_text().len(), |space_idx| space_idx + 1);

        let word = if next_word_boundary > self.width {
            &self.current_text()[..self.width]
        } else {
            &self.current_text()[..next_word_boundary]
        };

        self.pos += word.len();
        Some(word)
    }
}

impl<'a> WordSplitter<'a> {
    fn current_text(&self) -> &'a str {
        &self.text[self.pos..]
    }

    fn at_end(&self) -> bool {
        self.pos == self.text.len()
    }
}
