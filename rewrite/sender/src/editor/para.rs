use super::wrap;
use std::ops::Index;

#[derive(Debug, PartialEq)]
pub(super) struct Paragraph {
    lines: Vec<String>, // invariant: lines.len() is always >= 1
}

impl Default for Paragraph {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
        }
    }
}

impl Paragraph {
    pub(super) fn rewrap(&mut self, width: usize) {
        let wrapped = wrap(self.lines.iter().map(|s| s.as_str()), width);
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

            for _ in l.as_bytes() {
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

        let before = other_lines[0].drain(..column).collect();
        self.lines.push(before);

        Self { lines: other_lines }
    }

    pub(super) fn join(&mut self, mut p: Self) {
        self.lines.append(&mut p.lines);
    }

    pub(super) fn insert(&mut self, c: char, line: usize, column: usize) {
        self.lines[line].insert(column, c);
    }

    pub(super) fn remove(&mut self, line: usize, column: usize) {
        self.lines[line].remove(column);
    }

    pub(super) fn lines(&self) -> impl Iterator<Item = &str> {
        self.lines.iter().map(|l| l.as_str())
    }

    pub(super) fn num_lines(&self) -> usize {
        self.lines.len()
    }
}

impl Index<usize> for Paragraph {
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        &self.lines[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrap() {
        let mut para = Paragraph {
            lines: vec!["foo bar".to_string()],
        };

        para.rewrap(4);

        assert_eq!(
            para,
            Paragraph {
                lines: vec!["foo ".to_string(), "bar".to_string()]
            }
        );
    }

    #[test]
    fn idx_of_coords() {
        let para = Paragraph {
            lines: vec![
                "one".to_string(),
                "two".to_string(),
                "three".to_string(),
                "four".to_string(),
            ],
        };

        assert_eq!(para.idx_of_coords(3, 4), 15);
    }

    #[test]
    fn coords_of_idx() {
        let para = Paragraph {
            lines: vec!["foo".to_string(), "bar".to_string()],
        };

        assert_eq!(para.coords_of_idx(3), (0, 3));
    }

    #[test]
    fn coords_of_idx_0() {
        let para = Paragraph {
            lines: vec!["a".to_string()],
        };

        assert_eq!(para.coords_of_idx(0), (0, 0));
    }

    #[test]
    fn split_off() {
        let mut para = Paragraph {
            lines: vec![
                "foo".to_string(),
                "bar".to_string(),
                "baz".to_string(),
                "quux".to_string(),
            ],
        };

        assert_eq!(
            para.split_off(2, 1),
            Paragraph {
                lines: vec!["az".to_string(), "quux".to_string()]
            }
        );

        assert_eq!(
            para,
            Paragraph {
                lines: vec!["foo".to_string(), "bar".to_string(), "b".to_string()]
            }
        );
    }

    #[test]
    fn join() {
        let mut para1 = Paragraph {
            lines: vec!["alpha".to_string(), "beta".to_string()],
        };

        let para2 = Paragraph {
            lines: vec!["gamma".to_string(), "delta".to_string()],
        };

        para1.join(para2);
        assert_eq!(
            para1,
            Paragraph {
                lines: vec![
                    "alpha".to_string(),
                    "beta".to_string(),
                    "gamma".to_string(),
                    "delta".to_string()
                ]
            }
        );
    }
}
