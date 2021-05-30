use flume::Sender;
use never::Never;
use std::io::BufReader;
use std::net::{SocketAddr, TcpStream};

#[derive(Debug)]
pub struct Protocol {
    server_stream: BufReader<TcpStream>,
    event_tx: Sender<model::Event>,
}

impl Protocol {
    pub fn connect(address: SocketAddr, event_tx: Sender<model::Event>) -> anyhow::Result<Self> {
        let mut server_stream = TcpStream::connect(address)?;
        jsonl::write(&mut server_stream, &model::ConnectionKind::Viewer)?;

        Ok(Self {
            server_stream: BufReader::new(server_stream),
            event_tx,
        })
    }

    pub fn receive_events(mut self) -> anyhow::Result<Never> {
        let past_events: Vec<_> = jsonl::read(&mut self.server_stream)?;

        for event in past_events {
            self.event_tx.send(event).unwrap();
        }

        loop {
            let event = jsonl::read(&mut self.server_stream)?;
            self.event_tx.send(event).unwrap();
        }
    }
}
