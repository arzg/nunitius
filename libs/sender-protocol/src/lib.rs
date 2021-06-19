use jsonl::Connection;
use std::io::BufReader;
use std::marker::PhantomData;
use std::net::{SocketAddr, TcpStream};

#[derive(Debug)]
pub struct Protocol<S: State> {
    connection: Connection<BufReader<TcpStream>, TcpStream>,
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
        let stream = TcpStream::connect(address)?;
        let mut connection = Connection::new_from_tcp_stream(stream)?;

        connection.write(&model::ConnectionKind::Sender)?;

        Ok(Self {
            connection,
            _state: PhantomData,
        })
    }

    pub fn login(mut self, user: model::User) -> anyhow::Result<LoginResult> {
        self.connection.write(&model::SenderRequest::Login(user))?;

        let login_response = self.connection.read()?;

        let login_result = match login_response {
            model::LoginResponse::Succeeded => LoginResult::Succeeded(Protocol {
                connection: self.connection,
                _state: PhantomData,
            }),
            model::LoginResponse::Taken => LoginResult::Taken(self),
        };

        Ok(login_result)
    }
}

pub enum LoginResult {
    Succeeded(Protocol<SendingMessages>),
    Taken(Protocol<LoggingIn>),
}

impl Protocol<SendingMessages> {
    pub fn send_message(&mut self, message: model::Message) -> anyhow::Result<()> {
        let request = model::SenderRequest::NewMessage(message);
        self.connection.write(&request)?;

        Ok(())
    }
}
