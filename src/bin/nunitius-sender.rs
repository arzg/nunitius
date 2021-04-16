use jsonl::Connection;
use nunitius::{Color, ConnectionKind, LoginResponse, Message, User};
use std::io::{self, Write};
use std::net::TcpStream;

type TcpConnection = Connection<io::BufReader<TcpStream>, TcpStream>;

fn main() -> anyhow::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let stream = TcpStream::connect("127.0.0.1:9999")?;
    let mut connection = Connection::new_from_tcp_stream(stream)?;

    connection.write(&ConnectionKind::Sender)?;

    let nickname = login(&stdin, &mut stdout, &mut connection)?;

    loop {
        let input = read_input("Type a message", &stdin, &mut stdout)?;

        let input = if let Some(i) = input {
            i
        } else {
            continue;
        };

        let message = Message {
            body: input,
            author: nickname.clone(),
        };

        connection.write(&message)?;
    }
}

fn login(
    stdin: &io::Stdin,
    stdout: &mut io::Stdout,
    connection: &mut TcpConnection,
) -> anyhow::Result<User> {
    loop {
        let nickname = read_input("Choose a nickname", stdin, stdout)?;

        let nickname = if let Some(n) = nickname {
            n
        } else {
            continue;
        };

        let user = User {
            nickname: nickname.clone(),
            color: read_color(stdin, stdout)?,
        };

        connection.write(&user)?;

        let response: LoginResponse = connection.read()?;

        if response.nickname_taken {
            eprintln!("Nickname ‘{}’ taken. Try another one.", nickname);
        } else {
            return Ok(user);
        }
    }
}

fn read_color(stdin: &io::Stdin, stdout: &mut io::Stdout) -> anyhow::Result<Option<Color>> {
    loop {
        let color = if let Some(s) = read_input("Choose a color", stdin, stdout)? {
            s
        } else {
            return Ok(None);
        };

        let color = match color.as_str() {
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            _ => {
                eprintln!("‘{}’ is an invalid color.", color);
                continue;
            }
        };

        return Ok(Some(color));
    }
}

fn read_input(
    prompt: &str,
    stdin: &io::Stdin,
    stdout: &mut io::Stdout,
) -> anyhow::Result<Option<String>> {
    write!(stdout, "{} > ", prompt)?;
    stdout.flush()?;

    let mut input = String::new();
    stdin.read_line(&mut input)?;

    let input = input.trim().to_string();

    Ok(if input.is_empty() { None } else { Some(input) })
}
