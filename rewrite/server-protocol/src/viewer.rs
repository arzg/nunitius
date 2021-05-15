use std::marker::PhantomData;
use std::net::TcpStream;

#[derive(Debug)]
pub struct ViewerConnection<S: State> {
    stream: TcpStream,
    _state: PhantomData<S>,
}

pub trait State {}

#[derive(Debug)]
pub enum SendingPastEvents {}

#[derive(Debug)]
pub enum SendingEvents {}

impl State for SendingPastEvents {}
impl State for SendingEvents {}

impl ViewerConnection<SendingPastEvents> {
    pub(crate) fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _state: PhantomData,
        }
    }

    pub fn send_past_events(
        mut self,
        past_events: &[model::Event],
    ) -> anyhow::Result<ViewerConnection<SendingEvents>> {
        jsonl::write(&mut self.stream, &past_events)?;

        Ok(ViewerConnection {
            stream: self.stream,
            _state: PhantomData,
        })
    }
}

impl ViewerConnection<SendingEvents> {
    pub fn send_event(&mut self, event: model::Event) -> anyhow::Result<()> {
        jsonl::write(&mut self.stream, &event)?;
        Ok(())
    }
}
