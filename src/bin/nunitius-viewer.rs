use nunitius::{ConnectionKind, Event, Login, Message};
use std::io::BufReader;
use std::net::TcpStream;

fn main() -> anyhow::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:9999")?;

    jsonl::write(&mut stream, &ConnectionKind::Viewer)?;

    let mut stream = BufReader::new(stream);

    let existing_events: Vec<_> = jsonl::read(&mut stream)?;

    for event in existing_events {
        display_event(event);
    }

    loop {
        let event = jsonl::read(&mut stream)?;
        display_event(event);
    }
}

fn display_event(event: Event) {
    match event {
        Event::Message(Message { body, author }) => println!("{}: {}", author, body),
        Event::Login(Login { nickname }) => println!("Login with nickname {}", nickname),
        Event::Logout { nickname } => println!("{} logged out!", nickname),
    }
}
