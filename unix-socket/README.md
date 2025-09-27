# Unix Socket Communication in Rust

This project demonstrates basic inter-process communication (IPC) using Unix domain sockets in Rust. It includes a server that listens for incoming messages and a client that sends messages to the server. The server is designed to reverse the message received from the client and send it back as a response.

## Features

*   **Unix Domain Socket Server:** Listens for client connections and handles incoming messages.
*   **Unix Domain Socket Client:** Connects to the server and sends messages.
*   **Message Reversal:** The server reverses the received message before sending it back to the client.
*   **Single-shot Server Mode:** The server can be configured to handle a single client connection and then exit, useful for testing.

## Getting Started

These instructions will get you a copy of the project up and running on your local machine for development and testing purposes.

### Prerequisites

You will need to have Rust and Cargo installed. If you don't have them, you can install them using `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Building

Navigate to the project root directory and build the project using Cargo:

```bash
cargo build
```

This will compile both the server and client binaries.

## Usage

### Running the Server

The server listens on a Unix socket located at `/tmp/my_unix_socket.sock` by default.

```bash
cargo run --bin server
```

You should see output similar to:

```
Starting server on /tmp/my_unix_socket.sock
```

### Running the Client

You can send a message to the server using the client. If no message is provided, it defaults to "Hello from client!".

```bash
# Send a default message
cargo run --bin client

# Send a custom message
cargo run --bin client "Hello from the command line!"
```

Example output for `cargo run --bin client "Hello from the command line!"`:

```
Sending: Hello from the command line!
Received: !enil dnammoc eht morf olleH
```

The server's output will show the received message:

```
Received: Hello from the command line!
```

## Testing

To run the unit tests, which include a client-server communication test, use Cargo:

```bash
cargo test
```

The test verifies that the client can send a message and the server correctly reverses it and sends it back.
