use ui_types::StyledText;

pub trait View {
    type Extra;
    fn new(width: usize, height: usize, extra: Self::Extra) -> Self;

    fn render(&self) -> Vec<StyledText<'_>>;
    fn cursor(&self) -> (usize, usize);

    type Message: From<Input>;
    type Command;
    fn update(&mut self, message: Self::Message) -> StateChange<Self::Command>;
}

pub enum Input {
    Keypresses(String),
    Enter,
    Backspace,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Resize { width: usize, height: usize },
}

pub enum StateChange<Command> {
    Alive { command: Option<Command> },
    Dead,
}
