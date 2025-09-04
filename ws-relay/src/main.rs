use futures_util::{SinkExt, stream::StreamExt};
use log::{error, info, warn}; // import warn
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{Mutex, oneshot},
};
use tokio_tungstenite::accept_hdr_async;

// The following type aliases remain unchanged
type WebSocketStream = tokio_tungstenite::WebSocketStream<TcpStream>;
type PeerTx = oneshot::Sender<WebSocketStream>;
type SharedState = Arc<Mutex<HashMap<String, PeerTx>>>;

#[tokio::main]
async fn main() {
    env_logger::init();
    let addr = "0.0.0.0:8765";
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");
    info!("WebSocket Relay server started at: {}", addr);
    let state = SharedState::new(Mutex::new(HashMap::new()));
    while let Ok((stream, peer_addr)) = listener.accept().await {
        tokio::spawn(handle_connection(state.clone(), stream, peer_addr));
    }
}

async fn handle_connection(state: SharedState, stream: TcpStream, peer_addr: SocketAddr) {
    let mut path_from_req = None;
    let callback =
        |req: &tokio_tungstenite::tungstenite::handshake::server::Request,
         response: tokio_tungstenite::tungstenite::handshake::server::Response| {
            path_from_req = Some(req.uri().path().to_string());
            Ok(response)
        };

    let mut ws_stream = match accept_hdr_async(stream, callback).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("WebSocket handshake failed: {}, from: {}", e, peer_addr);
            return;
        }
    };

    let path = match path_from_req {
        Some(p) => p,
        None => {
            error!("Could not get path from request, from: {}", peer_addr);
            return;
        }
    };

    // --- Start of main logic modification ---
    // Parse the path, format should be "/[role]/[session_id]"
    let parts: Vec<&str> = path.trim_matches('/').split('/').collect();
    if parts.len() != 2 {
        warn!(
            "Invalid path format: '{}', from: {}. Should be /[role]/[session_id]",
            path, peer_addr
        );
        return;
    }
    let role = parts[0];
    let session_id = parts[1].to_string();

    info!(
        "New connection request: role='{}', SessionID='{}', from: {}",
        role, session_id, peer_addr
    );

    if role == "host" {
        // This is the connection logic for Host (B)
        let mut pending_hosts = state.lock().await;

        if pending_hosts.contains_key(&session_id) {
            warn!(
                "Host tried to connect to an already occupied Session ID: '{}'",
                session_id
            );
            // Can choose to disconnect this connection or notify the other party
            return;
        }

        info!(
            "Host '{}' is waiting for a Client connection...",
            session_id
        );
        let (peer_tx, peer_rx) = oneshot::channel();
        pending_hosts.insert(session_id.clone(), peer_tx);
        drop(pending_hosts);

        match peer_rx.await {
            Ok(peer_ws) => {
                info!(
                    "Client connected to '{}', pairing successful, starting data forwarding.",
                    session_id
                );
                forward_streams(ws_stream, peer_ws).await; // Note the parameter order, ws_stream is the host
                info!("Forwarding for Session '{}' has ended.", session_id);
            }
            Err(_) => {
                // If an error occurs before waiting for the Client, clean up its own record
                let mut pending_hosts = state.lock().await;
                pending_hosts.remove(&session_id);
                drop(pending_hosts);
                info!(
                    "Host '{}' disconnected or an error occurred while waiting, cleaned up.",
                    session_id
                );
            }
        }
    } else if role == "client" {
        // This is the connection logic for Client (C)
        let mut pending_hosts = state.lock().await;

        if let Some(peer_tx) = pending_hosts.remove(&session_id) {
            // Found the waiting Host, pairing successful
            info!(
                "Client found the waiting Host '{}', proceeding with pairing.",
                session_id
            );
            if peer_tx.send(ws_stream).is_err() {
                error!(
                    "Could not send Client connection to Host, maybe the Host just disconnected. Session ID: '{}'",
                    session_id
                );
            }
        } else {
            // Did not find the corresponding Host
            warn!(
                "Client tried to connect to a non-existent or unprepared Session ID: '{}'",
                session_id
            );
            // Disconnect this Client connection directly
            let _ = ws_stream.close(None).await;
        }
    } else {
        warn!("Unknown role: '{}', from: {}", role, peer_addr);
    }
    // --- End of main logic modification ---
}

// The forward_streams function remains unchanged, its design already ensures synchronous disconnection
async fn forward_streams(ws1: WebSocketStream, ws2: WebSocketStream) {
    let (mut write1, mut read1) = ws1.split();
    let (mut write2, mut read2) = ws2.split();

    loop {
        tokio::select! {
            Some(Ok(msg)) = read1.next() => {
                if write2.send(msg).await.is_err() {
                    break;
                }
            }
            Some(Ok(msg)) = read2.next() => {
                if write1.send(msg).await.is_err() {
                    break;
                }
            }
            else => {
                break;
            }
        }
    }
}
