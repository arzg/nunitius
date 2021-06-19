use sender_protocol::{LoggingIn, LoginResult, Protocol, SendingMessages};
use std::io::{self, Write};

fn main() -> anyhow::Result<()> {
    let protocol = Protocol::connect(([127, 0, 0, 1], 9292).into())?;

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut protocol = login(protocol, &stdin, &mut stdout)?;

    loop {
        let message = prompt("Type a message", &stdin, &mut stdout)?;
        protocol.send_message(model::Message { body: message })?;
    }
}

fn login(
    protocol: Protocol<LoggingIn>,
    stdin: &io::Stdin,
    stdout: &mut io::Stdout,
) -> anyhow::Result<Protocol<SendingMessages>> {
    let nickname = prompt("Choose a nickname", stdin, stdout)?;

    match protocol.login(model::User {
        nickname: nickname.clone(),
    })? {
        LoginResult::Succeeded(protocol) => {
            eprintln!("Successfully logged in as ‘{}’.", nickname);
            Ok(protocol)
        }
        LoginResult::Taken(protocol) => {
            eprintln!("Nickname ‘{}’ taken.", nickname);
            login(protocol, stdin, stdout)
        }
    }
}

fn prompt(prompt: &str, stdin: &io::Stdin, stdout: &mut io::Stdout) -> anyhow::Result<String> {
    write!(stdout, "{} > ", prompt)?;
    stdout.flush()?;

    let mut input = String::new();
    stdin.read_line(&mut input)?;

    Ok(input.trim().to_string())
}
