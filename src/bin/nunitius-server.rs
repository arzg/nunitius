use std::io::Read;
use std::net::TcpListener;

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:9999")?;

    for stream in listener.incoming() {
        let mut stream = stream?;

        let mut text_from_connection = Vec::new();
        stream.read_to_end(&mut text_from_connection)?;

        let text_from_connection = String::from_utf8_lossy(&text_from_connection);
        println!("{}", text_from_connection);
    }

    Ok(())
}
