use nunitius::{Event, Login, Message};
use std::io::BufReader;
use std::net::TcpListener;

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:9999")?;

    for stream in listener.incoming() {
        let stream = stream?;
        let mut stream = BufReader::new(stream);

        let event: Event = jsonl::read(&mut stream)?;

        match event {
            Event::Message(Message(message)) => println!("{}", message),
            Event::Login(Login { nickname }) => println!("Login with nickname {}", nickname),
        };
    }

    Ok(())
}
