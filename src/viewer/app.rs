use super::{ui, Event, Timeline};
use crate::User;
use std::collections::HashSet;

pub struct App {
    timeline: Timeline,
    currently_typing_users: HashSet<User>,
    terminal_height: usize,
}

impl App {
    pub fn new(terminal_height: usize) -> Self {
        Self {
            timeline: Timeline::new(terminal_height - 1),
            currently_typing_users: HashSet::new(),
            terminal_height,
        }
    }

    pub fn render(&self) -> RenderedUi {
        let mut output = RenderedUi::default();

        let visible_events = self.timeline.visible_events();
        for event in visible_events {
            output.add_line(&ui::render_event(event));
        }

        for _ in 0..self.terminal_height - visible_events.len() - 1 {
            output.add_empty_line();
        }

        output.add_line(&ui::render_currently_typing_users(
            self.currently_typing_users.iter(),
        ));

        output
    }

    pub fn handle_event(&mut self, event: Event) {
        self.timeline.add_event(event);
    }

    pub fn scroll_up(&mut self) {
        self.timeline.scroll_up();
    }

    pub fn scroll_down(&mut self) {
        self.timeline.scroll_down();
    }

    pub fn resize(&mut self, new_terminal_height: usize) {
        self.terminal_height = new_terminal_height;
        self.timeline.resize(new_terminal_height - 1);
    }

    pub fn start_typing(&mut self, user: User) {
        self.currently_typing_users.insert(user);
    }

    pub fn stop_typing(&mut self, user: &User) {
        self.currently_typing_users.remove(user);
    }
}

#[derive(Default)]
pub struct RenderedUi {
    buf: String,
    num_lines: usize,
}

impl RenderedUi {
    fn add_empty_line(&mut self) {
        self.add_line("");
    }

    fn add_line(&mut self, line: &str) {
        assert!(!line.contains('\n'));

        if self.num_lines != 0 {
            self.buf.push('\n');
        }

        self.buf.push_str(line);
        self.num_lines += 1;
    }

    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.buf.split('\n')
    }
}
