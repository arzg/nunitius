use crate::Event;
use flume::{Receiver, Selector};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io;
use std::net::TcpStream;
use tracing::{error, info, span, Level};

pub fn viewer_handler(event_rx: Receiver<Event>, viewer_rx: Receiver<TcpStream>) {
    let span = span!(Level::INFO, "handling_viewers");
    let _guard = span.enter();

    let viewers = RefCell::new(HashMap::new());
    let mut current_viewer_idx = 0;

    loop {
        Selector::new()
            .recv(&viewer_rx, |viewer| {
                info!("received new viewer");
                handle_new_viewer(&viewers, &mut current_viewer_idx, viewer.unwrap());
            })
            .recv(&event_rx, |event| {
                info!("received event");
                handle_new_event(&viewers, event.unwrap());
            })
            .wait();
    }
}

fn handle_new_viewer(
    viewers: &RefCell<HashMap<i32, TcpStream>>,
    current_viewer_idx: &mut i32,
    viewer: TcpStream,
) {
    viewers.borrow_mut().insert(*current_viewer_idx, viewer);
    *current_viewer_idx += 1;
}

fn handle_new_event(viewers: &RefCell<HashMap<i32, TcpStream>>, event: Event) {
    let mut closed_viewers = Vec::new();
    send_event_to_viewers(viewers, event, &mut closed_viewers);
    remove_closed_viewers(viewers, closed_viewers.into_iter());
}

fn send_event_to_viewers(
    viewers: &RefCell<HashMap<i32, TcpStream>>,
    event: Event,
    closed_viewers: &mut Vec<i32>,
) {
    let mut viewers = viewers.borrow_mut();

    for (idx, viewer) in viewers.iter_mut() {
        match jsonl::write(viewer, &event) {
            Ok(()) => info!("forwarded event to viewer"),

            Err(jsonl::WriteError::Io(io_error))
                if io_error.kind() == io::ErrorKind::BrokenPipe =>
            {
                info!("found closed viewer");
                closed_viewers.push(*idx);
            }

            Err(e) => error!("{:#}", anyhow::Error::new(e)),
        }
    }
}

fn remove_closed_viewers(
    viewers: &RefCell<HashMap<i32, TcpStream>>,
    closed_viewers: impl Iterator<Item = i32>,
) {
    let mut viewers = viewers.borrow_mut();

    for idx in closed_viewers {
        info!("removed closed viewer");

        let removed_viewer = viewers.remove(&idx);

        // we know the viewer we just removed
        // was present in the HashMap
        assert!(removed_viewer.is_some())
    }
}
