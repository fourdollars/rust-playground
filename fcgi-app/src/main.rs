use clap::Parser;
use std::path::PathBuf;
use std::os::unix::net::UnixListener;
use log::{info, error};
use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::net::SocketAddr;
use warp::Filter;
use http::header::HeaderMap;

fn capitalize_header_name(name: &str) -> String {
    name.to_lowercase()
        .split(&['-', '_'][..]) // Split by both '-' and '_'
        .map(|s| {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join("-") // Join with '-' for standard HTTP header format
}

fn get_hello_body(mut headers: Vec<(String, String)>) -> String {
    // Normalize and sort headers
    headers.sort_by(|a, b| a.0.cmp(&b.0));

    let mut response = String::from("Hello from Rust Server!\n\nReceived Headers:\n");
    for (name, value) in headers {
        let capitalized_name = capitalize_header_name(&name);
        response.push_str(&format!("{}: {}\n", capitalized_name, value));
    }
    response
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run as a standalone HTTP server
    #[arg(long)]
    http: Option<String>,
    /// Run as a FastCGI server over a local socket file (absolute path)
    #[arg(value_name = "SOCKET_PATH")]
    socket: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();

    if let Some(addr_str) = args.http {
        info!("Starting HTTP server on address: {}", addr_str);
        let addr: SocketAddr = match addr_str.parse() {
            Ok(a) => a,
            Err(e) => {
                error!("Failed to parse HTTP address {}: {}", addr_str, e);
                return;
            }
        };

        let hello = warp::path::end()
            .and(warp::header::headers_cloned())
            .map(|headers: HeaderMap| {
                let formatted_headers: Vec<(String, String)> = headers.iter()
                    .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("invalid").to_string()))
                    .collect();
                let body = get_hello_body(formatted_headers);
                warp::reply::with_header(body, "Content-Type", "text/plain")
            });

        warp::serve(hello).run(addr).await;
    } else if let Some(path) = args.socket {
        info!("Starting FastCGI server on socket: {:?}", path);
        // Remove the socket file if it already exists
        if path.exists() {
            if let Err(e) = std::fs::remove_file(&path) {
                error!("Failed to remove existing socket file {:?}: {}", path, e);
                return;
            }
        }

        let listener = match UnixListener::bind(&path) {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to bind to socket {:?}: {}", path, e);
                return;
            }
        };

        info!("FastCGI server listening on {:?}", path);

        // The `listener` variable must remain in scope for the duration of `fastcgi::run_raw`
        // to keep the file descriptor valid.
        fastcgi::run_raw(|mut req| {
            info!("Received FastCGI request.");
            let mut headers: Vec<(String, String)> = Vec::new();
            for (name, value) in req.params() {
                if name.starts_with("HTTP_") {
                    headers.push((name[5..].to_string(), value.to_string()));
                }
            }
            let body = get_hello_body(headers);
            let content = format!("Status: 200 OK\r\nContent-Type: text/plain\r\n\r\n{}", body);
            req.stdout().write_all(content.as_bytes()).unwrap_or_else(|e| {
                error!("Failed to write to stdout: {}", e);
            });
        }, listener.as_raw_fd());

        // Clean up the socket file when the server exits
        if let Err(e) = std::fs::remove_file(&path) {
            error!("Failed to remove socket file {:?}: {}", path, e);
        }

    } else {
        println!("Stdin mode is not implemented yet.");
    }
}
