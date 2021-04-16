use super::HistoryRequest;
use crate::Event;
use flume::{Receiver, Selector, Sender};
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

pub fn viewer_handler(
    viewer_rx: Receiver<TcpStream>,
    event_rx: Receiver<Event>,
    history_request_tx: Sender<HistoryRequest>,
) {
    let viewers = RefCell::new(HashMap::new());
    let mut viewer_id_generator = ViewerIdGenerator::default();

    loop {
        Selector::new()
            .recv(&viewer_rx, |viewer| {
                info!("received new viewer");

                if let Err(e) = handle_new_viewer(
                    viewer.unwrap(),
                    &viewers,
                    &mut viewer_id_generator,
                    &history_request_tx,
                ) {
                    error!("{:#}", e);
                }
            })
            .recv(&event_rx, |event| {
                info!("received event");
                handle_new_event(&viewers, event.unwrap());
            })
            .wait();
    }
}

fn handle_new_viewer(
    mut viewer: TcpStream,
    viewers: &RefCell<HashMap<ViewerId, TcpStream>>,
    viewer_id_generator: &mut ViewerIdGenerator,
    history_request_tx: &Sender<HistoryRequest>,
) -> anyhow::Result<()> {
    send_new_viewer_existing_history(&mut viewer, history_request_tx)?;

    let next_id = viewer_id_generator.next();
    viewers.borrow_mut().insert(next_id, viewer);

    Ok(())
}

fn send_new_viewer_existing_history(
    viewer: &mut TcpStream,
    history_request_tx: &Sender<HistoryRequest>,
) -> anyhow::Result<()> {
    let (history_tx, history_rx) = flume::bounded(0);

    history_request_tx
        .send(HistoryRequest { history_tx })
        .unwrap();
    info!("requested history");

    let history = history_rx.recv().unwrap();
    jsonl::write(viewer, &history)?;
    info!("sent history to viewer");

    Ok(())
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
