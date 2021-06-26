#[derive(Debug, PartialEq)]
pub enum StyledText<'a> {
    Bold(&'a str),
    Regular(&'a str),
    Red(&'a str),
}
