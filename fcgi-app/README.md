# `fcgi-app`: A Unified FastCGI and HTTP Server in Rust

`fcgi-app` is a simple Rust application that can operate as either a standalone HTTP server or a FastCGI application, demonstrating unified request handling and header display across both modes.

## Features

- **Standalone HTTP Server:** Listens on a specified address and port, responding to HTTP requests directly.
- **FastCGI Application:** Integrates with a FastCGI spawner (an external component) via a Unix socket, processing FastCGI requests.
- **Unified Request Handling:** Uses a common logic (`get_hello_body`) to generate responses, including client-sent HTTP headers, ensuring consistent output regardless of the operating mode.
- **Robust Header Normalization:** Automatically converts header names to a consistent capitalized format (e.g., `content-type` becomes `Content-Type`, `http_user_agent` becomes `Http-User-Agent`) and sorts them alphabetically for a clean, standardized display.
- **FastCGI Header Filtering:** In FastCGI mode, only client-sent HTTP headers (those prefixed with `HTTP_` in FastCGI parameters, plus `CONTENT_TYPE` and `CONTENT_LENGTH`) are extracted and displayed, ensuring that internal FastCGI parameters are not mistakenly presented as client headers.

## Building the Application

To build the `fcgi-app` project, navigate to the `fcgi-app` directory and run:

```bash
cargo build
```

This will compile the application and place the executable in `target/debug/fcgi-app` (or `target/release/fcgi-app` if you build with `--release`).

## Running the Application

### HTTP Mode

To run `fcgi-app` as a standalone HTTP server, specify the `--http` argument with the desired listening address and port. For example, to listen on `127.0.0.1:8080`:

```bash
target/debug/fcgi-app --http 127.0.0.1:8080
```

Then, you can access it with `curl` or your web browser:

```bash
curl http://127.0.0.1:8080/
```

### FastCGI Mode

To run `fcgi-app` as a FastCGI application, provide the path to the Unix socket file as a positional argument. This mode is typically used in conjunction with an external FastCGI spawner or a web server configured to proxy requests to the socket.

For example, if your spawner expects the socket at `/tmp/fcgi.sock`:

```bash
target/debug/fcgi-app /tmp/fcgi.sock
```

**Note:** An external FastCGI spawner (such as a Go-based one, or a web server like Nginx/Apache with FastCGI support) is responsible for managing the lifecycle of the FastCGI application, including starting it and cleaning up the socket file. You would typically configure your web server to forward requests to the spawner, which then communicates with the `fcgi-app` via the Unix socket.

## FastCGI Spawner Context

When `fcgi-app` runs in FastCGI mode, it expects to be launched and managed by an external FastCGI spawner. This spawner is responsible for:
- **Process Management:** Starting and stopping FastCGI application instances.
- **Socket Communication:** Establishing and managing the Unix socket connection with the `fcgi-app`.
- **Parameter Passing:** Converting incoming HTTP requests into FastCGI parameters (including client HTTP headers prefixed with `HTTP_`, and other standard FastCGI environment variables like `REQUEST_METHOD`, `SCRIPT_FILENAME`, etc.) and passing them to the `fcgi-app`.
- **Response Proxying:** Receiving FastCGI responses from `fcgi-app` and converting them back into HTTP responses for the client.

It's important to understand that the spawner sends a comprehensive set of FastCGI parameters to the `fcgi-app`, as defined by the FastCGI protocol. The `fcgi-app` then intelligently filters these to display only the relevant client-sent HTTP headers, ensuring a clean and unified output.