use std::io::{self, Write};
use std::net::UdpSocket;
use std::thread;

fn main() -> io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:8080")?;
    println!("Listening on 0.0.0.0:8080");

    let socket_send = socket.try_clone()?;

    thread::spawn(move || {
        let mut buf = [0; 65535];
        loop {
            match socket.recv_from(&mut buf) {
                Ok((len, src)) => {
                    let data = &buf[..len];
                    match std::str::from_utf8(data) {
                        Ok(s) => println!("Received from {}: {}", src, s),
                        Err(_) => println!("Received from {}: {:?}", src, data),
                    }
                }
                Err(e) => eprintln!("Receive error: {}", e),
            }
        }
    });

    loop {
        print!("Enter destination (ip:port message): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        if parts.len() != 2 {
            println!("Invalid input format. Use: ip:port message");
            continue;
        }

        let addr = parts[0];
        let message = parts[1];

        let dest_addr = if addr.starts_with('[') {
            addr.to_string()
        } else {
            continue;
        };

        match socket_send.send_to(message.as_bytes(), &dest_addr) {
            Ok(sent) => println!("Sent {} bytes to {}", sent, dest_addr),
            Err(e) => eprintln!("Send error: {}", e),
        }
    }
}
