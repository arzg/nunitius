use super::HistoryRequest;
use crate::Event;
use flume::{Receiver, Selector};
use log::info;
use std::cell::RefCell;

pub fn history_handler(event_rx: Receiver<Event>, request_rx: Receiver<HistoryRequest>) {
    let events = RefCell::new(Vec::new());

    loop {
        Selector::new()
            .recv(&event_rx, |event| {
                events.borrow_mut().push(event.unwrap());
                info!("added event to history");
            })
            .recv(&request_rx, |request| {
                let HistoryRequest { history_tx } = request.unwrap();
                let events = events.borrow().clone();
                history_tx.send(events).unwrap();
                info!("replied to request for history");
            })
            .wait();
    }
}
