use unix_socket::{run_server, DEFAULT_SOCKET_PATH};

fn main() {
    println!("Starting server on {}", DEFAULT_SOCKET_PATH);
    if let Err(e) = run_server(DEFAULT_SOCKET_PATH, false) {
        eprintln!("Server error: {}", e);
    }
}