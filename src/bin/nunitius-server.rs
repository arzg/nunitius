use nunitius::{Event, Login, Message};
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:9999")?;

    for stream in listener.incoming() {
        let stream = stream?;

        thread::spawn(|| {
            if let Err(e) = handle_connection(stream) {
                eprintln!("Error: {}", e);
            }
        });
    }

    Ok(())
}

fn handle_connection(stream: TcpStream) -> anyhow::Result<()> {
    let mut stream = BufReader::new(stream);

    loop {
        let event: Event = jsonl::read(&mut stream)?;

        match event {
            Event::Message(Message { body, author }) => println!("{}: {}", author, body),
            Event::Login(Login { nickname }) => println!("Login with nickname {}", nickname),
        };
    }
}
