use super::{Lines, Paragraph};
use text::Text;

pub(super) enum Renderer<'a, I>
where
    I: Iterator<Item = &'a Paragraph>,
{
    HasParas {
        paras: I,
        current_para_lines: Lines<'a>,
    },
    Empty,
}

impl<'a, I> Renderer<'a, I>
where
    I: Iterator<Item = &'a Paragraph>,
{
    pub(super) fn new(mut paras: I) -> Self {
        if let Some(para) = paras.next() {
            let current_para_lines = para.lines();

            Self::HasParas {
                paras,
                current_para_lines,
            }
        } else {
            Self::Empty
        }
    }
}

impl<'a, I> Iterator for Renderer<'a, I>
where
    I: Iterator<Item = &'a Paragraph>,
{
    type Item = Text<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let (paras, current_para_lines) = match self {
            Self::HasParas {
                paras,
                ref mut current_para_lines,
            } => (paras, current_para_lines),

            Self::Empty => return None,
        };

        if let Some(line) = current_para_lines.next() {
            return Some(line);
        }

        let para = paras.next()?;
        *current_para_lines = para.lines();

        // we’ve reached the end of the paragraph,
        // so we return an empty line,
        // since paragraph breaks are one line long
        Some(Text::default())
    }
}
