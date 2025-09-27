use clap::Parser;
use http::header::HeaderMap;
use log::{error, info};
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::net::SocketAddr;
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use warp::Filter;

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

fn generate_response_body(
    method: &str,
    path: &str,
    query: &str,
    remote_addr: &str,
    proto: &str,
    headers: Vec<(String, String)>,
) -> String {
    let mut body = String::new();

    body.push_str("--- Request Details ---\n");
    body.push_str(&format!("Method: {}\n", method));
    body.push_str(&format!("URL Path: {}\n", path));
    body.push_str(&format!("Query String: {}\n", query));
    body.push_str(&format!("Remote Address: {}\n", remote_addr));
    body.push_str(&format!("Protocol: {}\n", proto));

    body.push_str("\n--- HTTP Headers (from Request) ---\n");
    // BTreeMap will keep keys sorted.
    let mut sorted_headers = BTreeMap::new();
    for (name, value) in headers {
        // The Go http server canonicalizes header names.
        let canonical_name = capitalize_header_name(&name);
        sorted_headers
            .entry(canonical_name)
            .or_insert_with(Vec::new)
            .push(value);
    }

    for (name, values) in sorted_headers {
        for value in values {
            body.push_str(&format!("{}: {}\n", name, value));
        }
    }

    body.push_str("\n--- Process Environment Variables (std::env::vars()) ---\n");
    let mut env_vars: Vec<String> = std::env::vars()
        .map(|(key, value)| format!("{}={}", key, value))
        .collect();
    env_vars.sort();
    for env_var in env_vars {
        body.push_str(&format!("{}\n", env_var));
    }

    body
}

fn handle_fcgi_request(mut req: fastcgi::Request) {
    let params: HashMap<String, String> = req
        .params()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let method = params
        .get("REQUEST_METHOD")
        .map(|s| s.as_str())
        .unwrap_or("");
    let request_uri = params.get("REQUEST_URI").map(|s| s.as_str()).unwrap_or("");
    let (path, query) = if let Some(pos) = request_uri.find('?') {
        let (p, q) = request_uri.split_at(pos);
        (p, &q[1..])
    } else {
        (request_uri, "")
    };

    let remote_addr = params.get("REMOTE_ADDR").map(|s| s.as_str()).unwrap_or("");
    let remote_port = params.get("REMOTE_PORT").map(|s| s.as_str()).unwrap_or("");
    let full_remote_addr_owned = if !remote_addr.is_empty() && !remote_port.is_empty() {
        format!("{}:{}", remote_addr, remote_port)
    } else {
        remote_addr.to_string()
    };
    let full_remote_addr = full_remote_addr_owned.as_str();

    let proto = params
        .get("SERVER_PROTOCOL")
        .map(|s| s.as_str())
        .unwrap_or("");

    let mut headers: Vec<(String, String)> = Vec::new();
    for (name, value) in &params {
        if name.starts_with("HTTP_") {
            headers.push((name[5..].to_string(), value.to_string()));
        }
    }

    let body = generate_response_body(method, path, query, full_remote_addr, proto, headers);

    let content = format!("Status: 200 OK\r\nContent-Type: text/plain\r\n\r\n{}", body);
    req.stdout()
        .write_all(content.as_bytes())
        .unwrap_or_else(|e| {
            error!("Failed to write to stdout: {}", e);
        });
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

        let routes = warp::path::full()
            .and(warp::method())
            .and(warp::addr::remote())
            .and(warp::header::headers_cloned())
            .map(
                |path: warp::path::FullPath,
                 method: http::Method,
                 remote: Option<SocketAddr>,
                 headers: HeaderMap| {
                    let remote_addr = remote.map(|s| s.to_string()).unwrap_or_default();

                    let path_with_query = path.as_str();
                    let (path_part, query_part) = if let Some(pos) = path_with_query.find('?') {
                        let (p, q) = path_with_query.split_at(pos);
                        (p, &q[1..])
                    } else {
                        (path_with_query, "")
                    };

                    let formatted_headers: Vec<(String, String)> = headers
                        .iter()
                        .map(|(name, value)| {
                            (name.to_string(), value.to_str().unwrap_or("").to_string())
                        })
                        .collect();

                    let proto = "HTTP/1.1";

                    let body = generate_response_body(
                        method.as_str(),
                        path_part,
                        query_part,
                        &remote_addr,
                        proto,
                        formatted_headers,
                    );
                    warp::reply::with_header(body, "Content-Type", "text/plain")
                },
            );

        warp::serve(routes).run(addr).await;
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
        fastcgi::run_raw(
            |req| {
                info!("Received FastCGI request.");
                handle_fcgi_request(req);
            },
            listener.as_raw_fd(),
        );

        // Clean up the socket file when the server exits
        if let Err(e) = std::fs::remove_file(&path) {
            error!("Failed to remove socket file {:?}: {}", path, e);
        }
    } else {
        info!("Starting FastCGI server on stdin");
        fastcgi::run(|req| {
            handle_fcgi_request(req);
        });
    }
}
