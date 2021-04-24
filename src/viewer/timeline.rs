use super::Event;

pub struct Timeline {
    events: Vec<Event>,
    height: usize,
    top_event_idx: usize,
}

impl Timeline {
    pub fn new(height: usize) -> Self {
        Self {
            events: Vec::new(),
            height,
            top_event_idx: 0,
        }
    }

    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
        self.move_to_bottom();
    }

    pub fn visible_events(&self) -> &[Event] {
        let visible_events = &self.events[self.top_event_idx..self.bottom_event_idx()];

        let expected_num_events = if self.can_all_events_fit_on_screen() {
            self.events.len()
        } else {
            self.height
        };
        assert_eq!(visible_events.len(), expected_num_events);

        visible_events
    }

    fn move_to_bottom(&mut self) {
        while !self.at_bottom() {
            self.move_down();
        }
    }

    pub fn move_up(&mut self) {
        if !self.at_top() {
            self.top_event_idx -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if !self.at_bottom() {
            self.top_event_idx += 1;
        }
    }

    fn at_top(&self) -> bool {
        self.top_event_idx == 0
    }

    fn at_bottom(&self) -> bool {
        self.bottom_event_idx() == self.events.len()
    }

    fn bottom_event_idx(&self) -> usize {
        if self.can_all_events_fit_on_screen() {
            self.events.len()
        } else {
            self.top_event_idx + self.height
        }
    }

    fn can_all_events_fit_on_screen(&self) -> bool {
        self.events.len() <= self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::viewer::EventKind;
    use crate::User;
    use chrono::Utc;
    use once_cell::sync::Lazy;

    macro_rules! define_test_event {
        ($name:ident) => {
            static $name: Lazy<Event> = Lazy::new(|| Event {
                event: EventKind::Login,
                user: User {
                    nickname: stringify!($name).to_string(),
                    color: None,
                },
                time_occurred: Utc::now(),
            });
        };
    }

    define_test_event!(EVENT_1);
    define_test_event!(EVENT_2);
    define_test_event!(EVENT_3);
    define_test_event!(EVENT_4);

    #[test]
    fn empty_has_no_visible_events() {
        let timeline = Timeline::new(10);
        assert_eq!(timeline.visible_events(), []);
    }

    #[test]
    fn added_events_are_visible() {
        let mut timeline = Timeline::new(10);

        timeline.add_event(EVENT_1.clone());
        timeline.add_event(EVENT_2.clone());

        assert_eq!(
            timeline.visible_events(),
            [EVENT_1.clone(), EVENT_2.clone()]
        );
    }

    #[test]
    fn old_events_above_height_are_not_visible() {
        let mut timeline = Timeline::new(2);

        timeline.add_event(EVENT_1.clone());
        timeline.add_event(EVENT_2.clone());
        timeline.add_event(EVENT_3.clone());

        assert_eq!(
            timeline.visible_events(),
            [EVENT_2.clone(), EVENT_3.clone()]
        );
    }

    #[test]
    fn moving_up_reveals_old_events() {
        let mut timeline = Timeline::new(2);

        timeline.add_event(EVENT_1.clone());
        timeline.add_event(EVENT_2.clone());
        timeline.add_event(EVENT_3.clone());

        timeline.move_up();

        assert_eq!(
            timeline.visible_events(),
            [EVENT_1.clone(), EVENT_2.clone()]
        );
    }

    #[test]
    fn moving_up_past_top_does_nothing() {
        let mut timeline = Timeline::new(2);

        timeline.add_event(EVENT_1.clone());
        timeline.add_event(EVENT_2.clone());
        timeline.add_event(EVENT_3.clone());

        timeline.move_up();
        timeline.move_up();

        assert_eq!(
            timeline.visible_events(),
            [EVENT_1.clone(), EVENT_2.clone()]
        );
    }

    #[test]
    fn moving_down_reveals_newer_events() {
        let mut timeline = Timeline::new(2);

        timeline.add_event(EVENT_1.clone());
        timeline.add_event(EVENT_2.clone());
        timeline.add_event(EVENT_3.clone());

        timeline.move_up();
        assert_eq!(
            timeline.visible_events(),
            [EVENT_1.clone(), EVENT_2.clone()]
        );

        timeline.move_down();
        assert_eq!(
            timeline.visible_events(),
            [EVENT_2.clone(), EVENT_3.clone()]
        );
    }

    #[test]
    fn moving_down_past_bottom_does_nothing() {
        let mut timeline = Timeline::new(2);

        timeline.add_event(EVENT_1.clone());
        timeline.add_event(EVENT_2.clone());
        timeline.add_event(EVENT_3.clone());

        timeline.move_down();
        assert_eq!(
            timeline.visible_events(),
            [EVENT_2.clone(), EVENT_3.clone()]
        );
    }

    #[test]
    fn goes_to_the_bottom_after_adding_an_event() {
        let mut timeline = Timeline::new(2);

        timeline.add_event(EVENT_1.clone());
        timeline.add_event(EVENT_2.clone());
        timeline.add_event(EVENT_3.clone());
        assert_eq!(
            timeline.visible_events(),
            [EVENT_2.clone(), EVENT_3.clone()]
        );

        timeline.move_up();
        assert_eq!(
            timeline.visible_events(),
            [EVENT_1.clone(), EVENT_2.clone()]
        );

        timeline.add_event(EVENT_4.clone());
        assert_eq!(
            timeline.visible_events(),
            [EVENT_3.clone(), EVENT_4.clone()]
        );
    }
}
