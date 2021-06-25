mod split;

use crate::{Text, TextBuf};
use split::split_into_words;
use unicode_width::UnicodeWidthStr;

pub fn wrap(text: &Text<'_>, width: usize) -> Vec<TextBuf> {
    let mut lines = vec![TextBuf::default()];
    let mut current_line = 0;

    for word in split_into_words(text, width) {
        if lines[current_line].width() + word.width() > width {
            lines.push(word.into_text_buf());
            current_line += 1;
        } else {
            lines[current_line].push(word.as_str());
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check<const N: usize>(input: &str, width: usize, expected: [&str; N]) {
        let input = Text::new(input);
        let wrapped = wrap(&input, width);

        assert_eq!(
            wrapped,
            expected
                .iter()
                .map(|line| TextBuf::new(line.to_string()))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn keep_existing_text_the_same() {
        check("foo", 10, ["foo"]);
    }

    #[test]
    fn needed_width_same_as_width_of_input() {
        check("test", 4, ["test"]);
    }

    #[test]
    fn keep_spaces() {
        check("foo bar baz", 4, ["foo ", "bar ", "baz"]);
    }

    #[test]
    fn break_words_if_do_not_fit() {
        check(
            "longword short word",
            6,
            ["longwo", "rd ", "short ", "word"],
        );
    }

    #[test]
    fn wrap_based_on_width() {
        check("ab👍cdef😀😀test", 4, ["ab👍", "cdef", "😀😀", "test"]);
    }
}
