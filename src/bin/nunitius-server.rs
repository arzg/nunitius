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

    let event = jsonl::read(&mut stream)?;

    if let Event::Login(Login { nickname }) = event {
        println!("Login with nickname {}", nickname);
    } else {
        anyhow::bail!("expected connection to begin with login");
    }

    loop {
        let event = jsonl::read(&mut stream)?;

        match event {
            Event::Message(Message { body, author }) => println!("{}: {}", author, body),
            Event::Login(_) => anyhow::bail!("only expected one login each connection"),
        };
    }
}
