use unix_socket::{run_client, DEFAULT_SOCKET_PATH};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let message = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        "Hello from client!".to_string()
    };

    println!("Sending: {}", message);
    match run_client(DEFAULT_SOCKET_PATH, &message) {
        Ok(response) => println!("Received: {}", response),
        Err(e) => eprintln!("Client error: {}", e),
    }
}
