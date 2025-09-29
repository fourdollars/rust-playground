# fcgi-hello

A simple FastCGI application written in Rust.

This application demonstrates a basic FastCGI server that can be run in two modes:

1.  **Standard I/O Mode**: Listens for FastCGI requests on `stdin`.
2.  **Unix Socket Mode**: Listens for FastCGI requests on a specified Unix socket.

## Building

To build the application, use Cargo:

```sh
cargo build --release
```

The compiled binary will be located at `target/release/fcgi-hello`.

## Usage

### Standard I/O Mode

To run the application listening on `stdin`, execute it without any arguments:

```sh
./target/release/fcgi-hello
```

This is useful for FastCGI process managers that communicate over `stdin` and `stdout`.

### Unix Socket Mode

To run the application listening on a Unix socket, provide the path to the socket as the first argument:

```sh
./target/release/fcgi-hello /tmp/fcgi-hello.sock
```

The application will create and bind to the specified socket file. If the file already exists, it will be removed first.

### Example with a Spawner

This application is designed to be used with a FastCGI spawner like `spawn-fcgi` or a web server with FastCGI support.

For example, using `spawn-fcgi`:

```sh
spawn-fcgi -s /tmp/fcgi-hello.sock -- ./target/release/fcgi-hello /tmp/fcgi-hello.sock
```
