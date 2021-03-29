use nunitius::Message;
use std::io;
use std::net::TcpStream;

fn main() -> anyhow::Result<()> {
    let stdin = io::stdin();

    loop {
        let stream = TcpStream::connect("127.0.0.1:9999")?;

        let mut input = String::new();
        stdin.read_line(&mut input)?;
        let input_without_newline = &input[..input.len() - 1];

        jsonl::write(stream, &Message(input_without_newline.to_string()))?;
    }
}
