use nunitius::{Event, Login, Message};
use std::io::{self, Write};
use std::net::TcpStream;

fn main() -> anyhow::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut stream = TcpStream::connect("127.0.0.1:9999")?;

    let nickname = read_input("Choose a nickname", &stdin, &mut stdout)?;
    let login = Login { nickname };

    jsonl::write(&mut stream, &Event::Login(login))?;

    loop {
        let input = read_input("Type a message", &stdin, &mut stdout)?;
        let message = Message(input);
        jsonl::write(&mut stream, &Event::Message(message))?;
    }
}

fn read_input(prompt: &str, stdin: &io::Stdin, stdout: &mut io::Stdout) -> anyhow::Result<String> {
    write!(stdout, "{} > ", prompt)?;
    stdout.flush()?;

    let mut input = String::new();
    stdin.read_line(&mut input)?;

    Ok(input.trim().to_string())
}
