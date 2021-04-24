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
        self.scroll_to_bottom();
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

    pub fn resize(&mut self, new_height: usize) {
        self.height = new_height;

        if self.past_bottom() {
            self.scroll_to_bottom();
        }
    }

    pub fn scroll_up(&mut self) {
        if !self.at_top() {
            self.top_event_idx -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        if !self.at_bottom() {
            self.top_event_idx += 1;
        }
    }

    fn scroll_to_bottom(&mut self) {
        self.top_event_idx = if self.can_all_events_fit_on_screen() {
            0
        } else {
            self.events.len() - self.height
        };
    }

    fn past_bottom(&self) -> bool {
        self.top_event_idx + self.height > self.events.len()
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
    use super::super::dummy_events::*;
    use super::*;

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
    fn scrolling_up_reveals_old_events() {
        let mut timeline = Timeline::new(2);

        timeline.add_event(EVENT_1.clone());
        timeline.add_event(EVENT_2.clone());
        timeline.add_event(EVENT_3.clone());

        timeline.scroll_up();

        assert_eq!(
            timeline.visible_events(),
            [EVENT_1.clone(), EVENT_2.clone()]
        );
    }

    #[test]
    fn scrolling_up_past_top_does_nothing() {
        let mut timeline = Timeline::new(2);

        timeline.add_event(EVENT_1.clone());
        timeline.add_event(EVENT_2.clone());
        timeline.add_event(EVENT_3.clone());

        timeline.scroll_up();
        timeline.scroll_up();

        assert_eq!(
            timeline.visible_events(),
            [EVENT_1.clone(), EVENT_2.clone()]
        );
    }

    #[test]
    fn scrolling_down_reveals_newer_events() {
        let mut timeline = Timeline::new(2);

        timeline.add_event(EVENT_1.clone());
        timeline.add_event(EVENT_2.clone());
        timeline.add_event(EVENT_3.clone());

        timeline.scroll_up();
        assert_eq!(
            timeline.visible_events(),
            [EVENT_1.clone(), EVENT_2.clone()]
        );

        timeline.scroll_down();
        assert_eq!(
            timeline.visible_events(),
            [EVENT_2.clone(), EVENT_3.clone()]
        );
    }

    #[test]
    fn scrolling_down_past_bottom_does_nothing() {
        let mut timeline = Timeline::new(2);

        timeline.add_event(EVENT_1.clone());
        timeline.add_event(EVENT_2.clone());
        timeline.add_event(EVENT_3.clone());

        timeline.scroll_down();
        assert_eq!(
            timeline.visible_events(),
            [EVENT_2.clone(), EVENT_3.clone()]
        );
    }

    #[test]
    fn scrolls_to_the_bottom_after_adding_an_event() {
        let mut timeline = Timeline::new(2);

        timeline.add_event(EVENT_1.clone());
        timeline.add_event(EVENT_2.clone());
        timeline.add_event(EVENT_3.clone());
        assert_eq!(
            timeline.visible_events(),
            [EVENT_2.clone(), EVENT_3.clone()]
        );

        timeline.scroll_up();
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

    #[test]
    fn resizing_smaller_does_not_scroll() {
        let mut timeline = Timeline::new(3);

        timeline.add_event(EVENT_1.clone());
        timeline.add_event(EVENT_2.clone());
        timeline.add_event(EVENT_3.clone());
        timeline.add_event(EVENT_4.clone());

        assert_eq!(
            timeline.visible_events(),
            [EVENT_2.clone(), EVENT_3.clone(), EVENT_4.clone()]
        );

        timeline.resize(2);
        assert_eq!(
            timeline.visible_events(),
            [EVENT_2.clone(), EVENT_3.clone()]
        );
    }

    #[test]
    fn resizing_larger_does_not_scroll_if_unneeded() {
        let mut timeline = Timeline::new(2);

        timeline.add_event(EVENT_1.clone());
        timeline.add_event(EVENT_2.clone());
        timeline.add_event(EVENT_3.clone());

        timeline.scroll_up();
        assert_eq!(
            timeline.visible_events(),
            [EVENT_1.clone(), EVENT_2.clone()]
        );

        timeline.resize(3);
        assert_eq!(
            timeline.visible_events(),
            [EVENT_1.clone(), EVENT_2.clone(), EVENT_3.clone()]
        );
    }

    #[test]
    fn resizing_larger_scrolls_up_if_needed() {
        let mut timeline = Timeline::new(2);

        timeline.add_event(EVENT_1.clone());
        timeline.add_event(EVENT_2.clone());
        timeline.add_event(EVENT_3.clone());
        assert_eq!(
            timeline.visible_events(),
            [EVENT_2.clone(), EVENT_3.clone()]
        );

        timeline.resize(3);
        assert_eq!(
            timeline.visible_events(),
            [EVENT_1.clone(), EVENT_2.clone(), EVENT_3.clone()]
        );
    }
}
