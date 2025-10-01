use clap::Parser;
use http::request::Parts;
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::header::{HeaderName, HeaderValue};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use log::{error, info};
use std::collections::{BTreeMap, HashMap};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::net::{TcpListener, UnixListener};
use tokio_fastcgi::Requests;

// Re-implement the header capitalization logic from fcgi-app
fn capitalize_header_name(name: &str) -> String {
    name.to_lowercase()
        .split(&['-', '_'][..])
        .map(|s| {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join("-")
}

// Unified service function to handle requests from different sources.
async fn unified_service<B>(req: Request<B>) -> Result<Response<Full<Bytes>>, Infallible>
where
    B: hyper::body::Body,
{
    let remote_addr = req
        .extensions()
        .get::<String>()
        .cloned()
        .unwrap_or_else(|| "Unknown".to_string());

    let (parts, _body) = req.into_parts();
    let mut body_str = String::new();

    body_str.push_str("--- Request Details ---\r\n");
    body_str.push_str(&format!("Method: {}\r\n", parts.method));
    body_str.push_str(&format!("URI: {}\r\n", parts.uri));
    body_str.push_str(&format!("Version: {:?}\r\n", parts.version));
    body_str.push_str(&format!("Remote Address: {}\r\n", remote_addr));

    body_str.push_str("\r\n--- HTTP Headers ---\r\n");
    let mut sorted_headers = BTreeMap::new();
    for (name, value) in &parts.headers {
        let canonical_name = capitalize_header_name(name.as_str());
        sorted_headers
            .entry(canonical_name)
            .or_insert_with(Vec::new)
            .push(value.to_str().unwrap_or("").to_string());
    }

    for (name, values) in sorted_headers {
        for value in values {
            body_str.push_str(&format!("{}: {}\r\n", name, value));
        }
    }

    body_str.push_str("\r\n--- Process Environment Variables ---\r\n");
    let mut env_vars: Vec<String> = std::env::vars()
        .map(|(key, value)| format!("{}={}", key, value))
        .collect();
    env_vars.sort();
    for env_var in env_vars {
        body_str.push_str(&format!("{}\r\n", env_var));
    }

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain")
        .body(Full::new(Bytes::from(body_str)))
        .unwrap();

    Ok(response)
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    http: Option<String>,
    #[arg(value_name = "SOCKET_PATH")]
    socket: Option<PathBuf>,
}

async fn run_http(addr_str: String) {
    info!("Starting HTTP server on address: {}", addr_str);
    let addr: SocketAddr = match addr_str.parse() {
        Ok(a) => a,
        Err(e) => {
            error!("Failed to parse HTTP address {}: {}", addr_str, e);
            return;
        }
    };

    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind to TCP socket {}: {}", addr, e);
            return;
        }
    };

    loop {
        let (stream, remote_addr) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to accept connection: {}", e);
                continue;
            }
        };

        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            let service = service_fn(move |mut req: Request<Incoming>| {
                req.extensions_mut().insert(remote_addr.to_string());
                unified_service(req)
            });

            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                error!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn handle_fcgi_request<W>(request: tokio_fastcgi::Request<W>) -> Result<(), std::io::Error>
where
    W: tokio::io::AsyncWrite + Unpin,
{
    let mut params = HashMap::new();
    if let Some(pair) = request.params_iter() {
        for (k, v) in pair {
            params.insert(k.as_bytes().to_vec(), v.to_vec());
        }
    }

    let (http_parts, remote_addr) = fcgi_params_to_http_parts(&params);

    let mut req = Request::from_parts(http_parts, Full::new(Bytes::new()));
    req.extensions_mut().insert(remote_addr);

    let http_res = unified_service(req).await.unwrap();
    let (parts, body) = http_res.into_parts();
    let body_bytes = body.collect().await.unwrap().to_bytes();

    let mut stdout = request.get_stdout();
    let mut headers_str = format!("Status: {}\r\n", parts.status.as_u16());
    for (name, value) in parts.headers.iter() {
        headers_str.push_str(&format!(
            "{}: {}\r\n",
            name.as_str(),
            value.to_str().unwrap()
        ));
    }
    headers_str.push_str("\r\n");

    stdout.write(headers_str.as_bytes()).await.unwrap();
    stdout.write(&body_bytes).await.unwrap();
    Ok(())
}

async fn run_fcgi(socket_path: Option<PathBuf>) {
    if let Some(path) = socket_path {
        info!("Starting FastCGI server on socket: {:?}", path);
        if path.exists() {
            if let Err(e) = tokio::fs::remove_file(&path).await {
                error!("Failed to remove existing socket file {:?}: {}", path, e);
                return;
            }
        }
        let listener = match UnixListener::bind(&path) {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to bind to Unix socket {:?}: {}", path, e);
                return;
            }
        };

        loop {
            if let Ok((socket, _)) = listener.accept().await {
                tokio::spawn(async move {
                    let (reader, writer) = socket.into_split();
                    let mut requests = Requests::from_split_socket((reader, writer), 10, 10);
                    while let Ok(Some(request)) = requests.next().await {
                        if let Err(err) = handle_fcgi_request(request).await {
                            error!("Error processing FCGI request: {}", err);
                        }
                    }
                });
            }
        }
    } else {
        info!("Starting FastCGI server on stdin");
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        let mut requests = Requests::from_split_socket((stdin, stdout), 10, 10);
        while let Ok(Some(request)) = requests.next().await {
            if let Err(err) = handle_fcgi_request(request).await {
                error!("Error processing FCGI request: {}", err);
            }
        }
    }
}

fn fcgi_params_to_http_parts(params: &HashMap<Vec<u8>, Vec<u8>>) -> (Parts, String) {
    let method = params
        .get(&b"request_method"[..])
        .and_then(|v| std::str::from_utf8(v).ok())
        .unwrap_or("GET");
    let uri = params
        .get(&b"request_uri"[..])
        .and_then(|v| std::str::from_utf8(v).ok())
        .unwrap_or("/");
    let mut builder = Request::builder().method(method).uri(uri);

    if let Some(headers) = builder.headers_mut() {
        for (name, value) in params {
            if let Ok(key_str) = std::str::from_utf8(name) {
                if key_str.starts_with("http_") {
                    let header_name_str = key_str[5..].replace("_", "-");
                    let header_name_str = capitalize_header_name(&header_name_str);

                    if let Ok(header_name) = HeaderName::from_bytes(header_name_str.as_bytes()) {
                        if let Ok(header_value) = HeaderValue::from_bytes(value) {
                            headers.insert(header_name, header_value);
                        }
                    }
                }
            }
        }
    }

    let remote_addr_str = format!(
        "{}:{}",
        params
            .get(&b"remote_addr"[..])
            .and_then(|v| std::str::from_utf8(v).ok())
            .unwrap_or(""),
        params
            .get(&b"remote_port"[..])
            .and_then(|v| std::str::from_utf8(v).ok())
            .unwrap_or("")
    );

    let (parts, _) = builder.body(()).unwrap().into_parts();
    info!(
        "HTTP Parts generated from FCGI: method={:?}, uri={:?}, headers={:?}",
        parts.method, parts.uri, parts.headers
    );
    (parts, remote_addr_str)
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args = Args::parse();

    if let Some(addr_str) = args.http {
        run_http(addr_str).await;
    } else {
        run_fcgi(args.socket).await;
    }
}
