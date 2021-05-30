use std::marker::PhantomData;
use std::net::{SocketAddr, TcpStream};

#[derive(Debug)]
pub struct Protocol<S: State> {
    stream: TcpStream,
    _state: PhantomData<S>,
}

pub trait State {}

#[derive(Debug)]
pub enum LoggingIn {}

#[derive(Debug)]
pub enum SendingMessages {}

impl State for LoggingIn {}
impl State for SendingMessages {}

impl Protocol<LoggingIn> {
    pub fn connect(address: SocketAddr) -> anyhow::Result<Self> {
        let mut stream = TcpStream::connect(address)?;
        jsonl::write(&mut stream, &model::ConnectionKind::Sender)?;

        Ok(Self {
            stream,
            _state: PhantomData,
        })
    }

    pub fn log_in(mut self, user: model::User) -> anyhow::Result<Protocol<SendingMessages>> {
        jsonl::write(&mut self.stream, &user)?;

        Ok(Protocol {
            stream: self.stream,
            _state: PhantomData,
        })
    }
}

impl Protocol<SendingMessages> {
    pub fn send_message(&mut self, message: model::Message) -> anyhow::Result<()> {
        let event_payload = model::EventPayload::Message(message);
        jsonl::write(&mut self.stream, &event_payload)?;

        Ok(())
    }
}
