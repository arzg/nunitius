use nunitius::Message;
use std::io::BufReader;
use std::net::TcpListener;

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:9999")?;

    for stream in listener.incoming() {
        let stream = stream?;
        let mut stream = BufReader::new(stream);

        let message: Message = jsonl::read(&mut stream)?;
        println!("{}", message.0);
    }

    Ok(())
}
