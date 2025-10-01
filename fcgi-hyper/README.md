# `fcgi-hyper`: A Unified FastCGI and HTTP Server with Hyper

`fcgi-hyper` is a Rust application that demonstrates how to handle web requests using the `hyper` library and serve them over both a standalone HTTP server and a FastCGI interface. This approach allows for unified application logic across different deployment environments.

## Features

- **Standalone HTTP Server:** Listens on a specified address and port, handling HTTP requests directly using `hyper`.
- **FastCGI Application (Unix Socket):** Listens on a Unix socket for requests from a FastCGI spawner.
- **FastCGI Application (Standard Input):** Reads FastCGI requests from standard input, suitable for process managers.
- **Unified Request Handling Logic:** The core request processing logic is now fully unified into a single `hyper`-compatible service function (`unified_service`). This function is responsible for generating the detailed response, regardless of whether the request originated from the HTTP server or the FastCGI interface.
- **Request Translation:** Incoming FastCGI requests are translated into standard `http::Request` objects before being passed to the unified application logic. The resulting `http::Response` is then translated back into a FastCGI response.
- **Detailed Response:** Provides a detailed response including request information, headers, and all process environment variables.

## Building the Application

To build the `fcgi-hyper` project, navigate to the `fcgi-hyper` directory and run:

```bash
cargo build
```

For a release build, use:

```bash
cargo build --release
```

The executable will be located at `target/debug/fcgi-hyper` or `target/release/fcgi-hyper`.

## Running the Application

### HTTP Mode

To run as a standalone HTTP server, use the `--http` flag with a listening address and port:

```bash
target/debug/fcgi-hyper --http 127.0.0.1:8080
```

You can then access it using `curl`:

```bash
curl http://127.0.0.1:8080/some/path?query=123
```

### FastCGI Mode (Unix Socket)

To run as a FastCGI application over a Unix socket, provide the absolute path to the socket file as an argument. This requires an external spawner (like `spawn-fcgi` or a web server) to send requests.

```bash
target/debug/fcgi-hyper /tmp/fcgi-hyper.sock
```

Example with `spawn-fcgi`:

```bash
spawn-fcgi -s /tmp/fcgi-hyper.sock -- ./target/release/fcgi-hyper /tmp/fcgi-hyper.sock
```

### FastCGI Mode (Standard Input/Output)

To run in standard I/O mode, execute the application without any arguments. This is typically used when a process manager handles the communication.

```bash
target/debug/fcgi-hyper
```