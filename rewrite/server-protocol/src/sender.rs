use std::io::BufReader;
use std::marker::PhantomData;
use std::net::TcpStream;

#[derive(Debug)]
pub struct SenderConnection<S: State> {
    stream: BufReader<TcpStream>,
    _state: PhantomData<S>,
}

pub trait State {}

#[derive(Debug)]
pub enum LoggingIn {}

#[derive(Debug)]
pub enum ReceivingEvents {}

impl State for LoggingIn {}
impl State for ReceivingEvents {}

impl SenderConnection<LoggingIn> {
    pub(crate) fn new(stream: TcpStream) -> Self {
        Self {
            stream: BufReader::new(stream),
            _state: PhantomData,
        }
    }

    pub fn receive_log_in(
        mut self,
    ) -> anyhow::Result<(SenderConnection<ReceivingEvents>, model::User)> {
        let user = jsonl::read(&mut self.stream)?;

        Ok((
            SenderConnection {
                stream: self.stream,
                _state: PhantomData,
            },
            user,
        ))
    }
}

impl SenderConnection<ReceivingEvents> {
    pub fn receive_event_payload(&mut self) -> anyhow::Result<model::EventPayload> {
        let event_payload = jsonl::read(&mut self.stream)?;
        Ok(event_payload)
    }
}
