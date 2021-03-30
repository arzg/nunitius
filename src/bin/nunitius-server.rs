use flume::{Receiver, Selector, Sender};
use nunitius::{ConnectionKind, Event, Login, Message};
use std::cell::RefCell;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:9999")?;

    let (viewer_tx, viewer_rx) = flume::bounded(100);
    let (events_tx, events_rx) = flume::bounded(100);

    thread::spawn(|| viewer_handler(events_rx, viewer_rx));

    for stream in listener.incoming() {
        let stream = stream?;
        let viewer_tx = viewer_tx.clone();
        let events_tx = events_tx.clone();

        thread::spawn(|| {
            if let Err(e) = handle_connection(stream, viewer_tx, events_tx) {
                eprintln!("Error: {}", e);
            }
        });
    }

    Ok(())
}

fn handle_connection(
    stream: TcpStream,
    viewer_tx: Sender<TcpStream>,
    events_tx: Sender<Event>,
) -> anyhow::Result<()> {
    let mut stream = BufReader::new(stream);

    let connection_kind: ConnectionKind = jsonl::read(&mut stream)?;

    match connection_kind {
        ConnectionKind::Sender => {
            let login: Login = jsonl::read(&mut stream)?;
            events_tx.send(Event::Login(login)).unwrap();

            loop {
                let message: Message = jsonl::read(&mut stream)?;
                events_tx.send(Event::Message(message)).unwrap();
            }
        }
        ConnectionKind::Viewer => viewer_tx.send(stream.into_inner()).unwrap(),
    }

    Ok(())
}

fn viewer_handler(events_rx: Receiver<Event>, viewer_rx: Receiver<TcpStream>) {
    let viewers = RefCell::new(Vec::new());

    loop {
        Selector::new()
            .recv(&viewer_rx, |viewer| {
                viewers.borrow_mut().push(viewer.unwrap());
            })
            .recv(&events_rx, |event| {
                let event = event.unwrap();
                for viewer in viewers.borrow_mut().iter_mut() {
                    if let Err(e) = jsonl::write(viewer, &event) {
                        eprintln!("Error: {}", anyhow::Error::new(e));
                    }
                }
            })
            .wait();
    }
}
