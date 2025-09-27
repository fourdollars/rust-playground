use std::os::unix::net::{UnixListener, UnixStream};
use std::io::{Read, Write};
use std::fs;

pub const DEFAULT_SOCKET_PATH: &str = "/tmp/my_unix_socket.sock";

/// Runs the Unix socket server.
/// If `single_shot` is true, the server will exit after handling one client.
pub fn run_server(socket_path: &str, single_shot: bool) -> std::io::Result<()> {
    if fs::metadata(socket_path).is_ok() {
        fs::remove_file(socket_path)?;
    }

    let listener = UnixListener::bind(socket_path)?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Handle client in a new thread for the main server,
                // but directly for the single-shot test to ensure completion.
                if single_shot {
                    handle_client(stream);
                    break; // Exit after one client
                } else {
                    std::thread::spawn(move || {
                        handle_client(stream);
                    });
                }
            }
            Err(err) => {
                eprintln!("Error accepting connection: {}", err);
                break;
            }
        }
    }
    Ok(())
}

fn handle_client(mut stream: UnixStream) {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer).unwrap();
    let received_message = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
    println!("Received: {}", received_message);

    let response = received_message.chars().rev().collect::<String>();
    stream.write_all(response.as_bytes()).unwrap();}

pub fn run_client(socket_path: &str, message: &str) -> std::io::Result<String> {
    let mut stream = UnixStream::connect(socket_path)?;
    stream.write_all(message.as_bytes())?;

    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;
    let response = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_client_server_communication() {
        let test_socket_path = "/tmp/test_socket.sock";

        // Run server in a separate thread in single-shot mode.
        let server_thread = thread::spawn(move || {
            run_server(test_socket_path, true).unwrap();
        });

        // Give the server a moment to start up.
        thread::sleep(Duration::from_millis(100));

        // Run client.
        let message = "Hello from test client!";
        let response = run_client(test_socket_path, message).unwrap();

        assert_eq!(response, "!tneilc tset morf olleH");

        // Wait for the server thread to finish.
        server_thread.join().unwrap();
    }
}
