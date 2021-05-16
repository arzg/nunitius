use std::ops::Index;
use text::{Text, TextBuf};

#[derive(Debug, PartialEq)]
pub(super) struct Paragraph {
    lines: Vec<TextBuf>, // invariant: lines.len() is always >= 1
}

impl Default for Paragraph {
    fn default() -> Self {
        Self {
            lines: vec![TextBuf::default()],
        }
    }
}

impl Paragraph {
    pub(super) fn rewrap(&mut self, width: usize) {
        let joined: String = self.lines.iter().map(TextBuf::as_str).collect();
        let joined = Text::new(&joined);
        let wrapped = crate::wrap(joined, width);

        self.lines = wrapped;
    }

    pub(super) fn idx_of_coords(&self, line: usize, column: usize) -> usize {
        let num_bytes_before_line: usize = self.lines[..line].iter().map(|l| l.len()).sum();
        num_bytes_before_line + column
    }

    pub(super) fn coords_of_idx(&self, idx: usize) -> (usize, usize) {
        let mut line = 0;
        let mut column = 0;
        let mut num_stepped = 0;

        'outer: for l in self.lines.iter() {
            if num_stepped == idx {
                break 'outer;
            }

            for _ in 0..l.len() {
                num_stepped += 1;
                column += 1;

                if num_stepped == idx {
                    break 'outer;
                }
            }

            line += 1;
            column = 0;
        }

        (line, column)
    }

    pub(super) fn split_off(&mut self, line: usize, column: usize) -> Self {
        let mut other_lines = self.lines.split_off(line);

        let (before, after) = other_lines[0].split(column);
        self.lines.push(before.into_text_buf());
        other_lines[0] = after.into_text_buf();

        Self { lines: other_lines }
    }

    pub(super) fn join(&mut self, mut p: Self) {
        self.lines.append(&mut p.lines);
    }

    pub(super) fn insert(&mut self, s: &str, line: usize, column: usize) {
        self.lines[line].insert(column, s);
    }

    pub(super) fn remove(&mut self, line: usize, column: usize) {
        self.lines[line].remove(column);
    }

    pub(super) fn lines(&self) -> Lines<'_> {
        Lines::new(self)
    }

    pub(super) fn num_lines(&self) -> usize {
        self.lines.len()
    }
}

impl Index<usize> for Paragraph {
    type Output = TextBuf;

    fn index(&self, index: usize) -> &Self::Output {
        &self.lines[index]
    }
}

pub(super) struct Lines<'a> {
    para: &'a Paragraph,
    line_idx: usize,
}

impl<'a> Lines<'a> {
    fn new(para: &'a Paragraph) -> Self {
        Self { para, line_idx: 0 }
    }
}

impl<'a> Iterator for Lines<'a> {
    type Item = Text<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.line_idx >= self.para.num_lines() {
            return None;
        }

        let line = &self.para[self.line_idx];
        self.line_idx += 1;

        Some(line.as_text())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! para {
        ($($lines:literal),*) => {
            Paragraph {
                lines: [$($lines),*]
                    .iter()
                    .map(|line| TextBuf::new(line.to_string()))
                    .collect(),
            }
        };
    }

    #[test]
    fn rewrap() {
        let mut para = para!("fo😀obar");

        para.rewrap(4);

        assert_eq!(para, para!("fo😀", "obar"));
    }

    #[test]
    fn idx_of_coords() {
        let para = para!("one", "two", "three", "four");
        assert_eq!(para.idx_of_coords(3, 4), 15);
    }

    #[test]
    fn coords_of_idx() {
        let para = para!("foo", "bar");
        assert_eq!(para.coords_of_idx(3), (0, 3));
    }

    #[test]
    fn coords_of_idx_0() {
        let para = para!("a");
        assert_eq!(para.coords_of_idx(0), (0, 0));
    }

    #[test]
    fn split_off() {
        let mut para = para!("foo", "bar", "baz", "quux");

        assert_eq!(para.split_off(2, 1), para!("az", "quux"));
        assert_eq!(para, para!("foo", "bar", "b"));
    }

    #[test]
    fn join() {
        let mut para1 = para!("alpha", "beta");
        let para2 = para!("gamma", "delta");

        para1.join(para2);
        assert_eq!(para1, para!("alpha", "beta", "gamma", "delta"));
    }
}
