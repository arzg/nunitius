use super::{Event, ServerEvent};
use crate::ConnectionKind;
use flume::Sender;
use std::io::BufReader;
use std::net::{TcpStream, ToSocketAddrs};

pub struct Protocol<S: ProtocolState>(S);

pub trait ProtocolState {}
impl ProtocolState for SendingConnectionKind {}
impl ProtocolState for ReadingHistory {}
impl ProtocolState for ReadingEvents {}

pub struct SendingConnectionKind {
    stream: TcpStream,
}

impl Protocol<SendingConnectionKind> {
    pub fn connect(addr: impl ToSocketAddrs) -> anyhow::Result<Self> {
        Ok(Self(SendingConnectionKind {
            stream: TcpStream::connect(addr)?,
        }))
    }

    pub fn send_connection_kind(
        mut self,
        server_event_tx: Sender<ServerEvent>,
        event_tx: Sender<Event>,
    ) -> anyhow::Result<Protocol<ReadingHistory>> {
        jsonl::write(&mut self.0.stream, &ConnectionKind::Viewer)?;

        Ok(Protocol(ReadingHistory {
            stream: BufReader::new(self.0.stream),
            server_event_tx,
            event_tx,
        }))
    }
}

pub struct ReadingHistory {
    stream: BufReader<TcpStream>,
    server_event_tx: Sender<ServerEvent>,
    event_tx: Sender<Event>,
}

impl Protocol<ReadingHistory> {
    pub fn read_history(mut self) -> anyhow::Result<Protocol<ReadingEvents>> {
        let history: Vec<ServerEvent> = jsonl::read(&mut self.0.stream)?;

        for server_event in history {
            self.0.server_event_tx.send(server_event.clone()).unwrap();

            if let Some(event) = Event::from_server_event(server_event) {
                self.0.event_tx.send(event).unwrap();
            }
        }

        Ok(Protocol(ReadingEvents {
            stream: self.0.stream,
            server_event_tx: self.0.server_event_tx,
            event_tx: self.0.event_tx,
        }))
    }
}

pub struct ReadingEvents {
    stream: BufReader<TcpStream>,
    server_event_tx: Sender<ServerEvent>,
    event_tx: Sender<Event>,
}

impl Protocol<ReadingEvents> {
    pub fn read_events(&mut self) -> anyhow::Result<Never> {
        loop {
            let server_event: ServerEvent = jsonl::read(&mut self.0.stream)?;

            self.0.server_event_tx.send(server_event.clone()).unwrap();

            if let Some(event) = Event::from_server_event(server_event) {
                self.0.event_tx.send(event).unwrap();
            }
        }
    }
}

pub enum Never {}
