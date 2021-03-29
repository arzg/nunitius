use nunitius::{Event, Login, Message};
use std::io::{self, Write};
use std::net::TcpStream;

fn main() -> anyhow::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let nickname = read_input("Choose a nickname", &stdin, &mut stdout)?;
    let login = Login { nickname };

    let stream = TcpStream::connect("127.0.0.1:9999")?;
    jsonl::write(stream, &Event::Login(login))?;

    loop {
        let stream = TcpStream::connect("127.0.0.1:9999")?;

        let input = read_input("Type a message", &stdin, &mut stdout)?;
        let message = Message(input);
        jsonl::write(stream, &Event::Message(message))?;
    }
}

fn read_input(prompt: &str, stdin: &io::Stdin, stdout: &mut io::Stdout) -> anyhow::Result<String> {
    write!(stdout, "{} > ", prompt)?;
    stdout.flush()?;

    let mut input = String::new();
    stdin.read_line(&mut input)?;

    Ok(input.trim().to_string())
}
