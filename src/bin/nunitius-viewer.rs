use nunitius::{ConnectionKind, Event, Login, Message};
use std::io::BufReader;
use std::net::TcpStream;

fn main() -> anyhow::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:9999")?;

    jsonl::write(&mut stream, &ConnectionKind::Viewer)?;

    let mut stream = BufReader::new(stream);

    loop {
        let event: Event = jsonl::read(&mut stream)?;

        match event {
            Event::Message(Message { body, author }) => println!("{}: {}", author, body),
            Event::Login(Login { nickname }) => println!("Login with nickname {}", nickname),
        }
    }
}
