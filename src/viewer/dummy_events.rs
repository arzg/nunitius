use super::{Event, EventKind};
use crate::User;
use chrono::Utc;
use once_cell::sync::Lazy;

macro_rules! define_dummy_event {
    ($name:ident) => {
        pub(super) static $name: Lazy<Event> = Lazy::new(|| Event {
            event: EventKind::Login,
            user: User {
                nickname: stringify!($name).to_string(),
                color: None,
            },
            time_occurred: Utc::now(),
        });
    };
}

define_dummy_event!(EVENT_1);
define_dummy_event!(EVENT_2);
define_dummy_event!(EVENT_3);
define_dummy_event!(EVENT_4);
