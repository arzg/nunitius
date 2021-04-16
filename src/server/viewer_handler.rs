use crate::Event;
use flume::{Receiver, Selector};
use log::{error, info};
use std::cell::RefCell;
use std::collections::HashMap;
use std::net::TcpStream;
use std::{io, mem};

#[derive(Default)]
struct ViewerIdGenerator {
    current: ViewerId,
}

impl ViewerIdGenerator {
    fn next(&mut self) -> ViewerId {
        let new_id = ViewerId(self.current.0 + 1);
        mem::replace(&mut self.current, new_id)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
struct ViewerId(u32);

pub fn viewer_handler(event_rx: Receiver<Event>, viewer_rx: Receiver<TcpStream>) {
    let viewers = RefCell::new(HashMap::new());
    let mut viewer_id_generator = ViewerIdGenerator::default();

    loop {
        Selector::new()
            .recv(&viewer_rx, |viewer| {
                info!("received new viewer");
                handle_new_viewer(&viewers, &mut viewer_id_generator, viewer.unwrap());
            })
            .recv(&event_rx, |event| {
                info!("received event");
                handle_new_event(&viewers, event.unwrap());
            })
            .wait();
    }
}

fn handle_new_viewer(
    viewers: &RefCell<HashMap<ViewerId, TcpStream>>,
    viewer_id_generator: &mut ViewerIdGenerator,
    viewer: TcpStream,
) {
    let next_id = viewer_id_generator.next();
    viewers.borrow_mut().insert(next_id, viewer);
}

fn handle_new_event(viewers: &RefCell<HashMap<ViewerId, TcpStream>>, event: Event) {
    let mut closed_viewers = Vec::new();
    send_event_to_viewers(viewers, event, &mut closed_viewers);
    remove_closed_viewers(viewers, closed_viewers.into_iter());
}

fn send_event_to_viewers(
    viewers: &RefCell<HashMap<ViewerId, TcpStream>>,
    event: Event,
    closed_viewers: &mut Vec<ViewerId>,
) {
    let mut viewers = viewers.borrow_mut();

    for (id, viewer) in viewers.iter_mut() {
        match jsonl::write(viewer, &event) {
            Ok(()) => info!("forwarded event to viewer"),

            Err(jsonl::WriteError::Io(io_error))
                if io_error.kind() == io::ErrorKind::BrokenPipe =>
            {
                info!("found closed viewer");
                closed_viewers.push(*id);
            }

            Err(e) => error!("{:#}", anyhow::Error::new(e)),
        }
    }
}

fn remove_closed_viewers(
    viewers: &RefCell<HashMap<ViewerId, TcpStream>>,
    closed_viewers: impl Iterator<Item = ViewerId>,
) {
    let mut viewers = viewers.borrow_mut();

    for id in closed_viewers {
        info!("removed closed viewer");

        let removed_viewer = viewers.remove(&id);

        // we know the viewer we just removed
        // was present in the HashMap
        assert!(removed_viewer.is_some())
    }
}
