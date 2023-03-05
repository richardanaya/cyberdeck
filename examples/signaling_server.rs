use anyhow::Result;
use axum::{response::Html, response::IntoResponse, routing::get, routing::post, Json, Router};
use cyberdeck::*;
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/connect", post(connect));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Running server on http://localhost:3000 ...");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn connect(Json(offer): Json<String>) -> impl IntoResponse {
    match start_peer_connection(offer).await {
        Ok(answer) => Ok(Json(answer)),
        Err(_) => Err("failed to connect"),
    }
}

async fn start_peer_connection(offer: String) -> Result<String> {
    let mut peer = Peer::new(move |e| async move {
        match e.data {
            PeerEventData::DataChannelMessage(c, m) => {
                println!(
                    "{}::Recieved a message from channel {} with id {}!",
                    e.peer_id,
                    c.label(),
                    c.id()
                );
                let msg_str = String::from_utf8(m.data.to_vec()).unwrap();
                println!(
                    "{}::Message from DataChannel '{}': {}",
                    e.peer_id,
                    c.label(),
                    msg_str
                );
            }
            PeerEventData::DataChannelStateChange(c) => {
                if c.ready_state() == RTCDataChannelState::Open {
                    println!("{}::DataChannel '{}'", e.peer_id, c.label());
                    c.send_text("Connected to client!".to_string())
                        .await
                        .unwrap();
                } else if c.ready_state() == RTCDataChannelState::Closed {
                    println!("{}::DataChannel '{}'", e.peer_id, c.label());
                }
            }
            PeerEventData::PeerConnectionStateChange(s) => {
                println!("{}::Peer connection state: {} ", e.peer_id, s)
            }
        }
    })
    .await?;
    let answer = peer.receive_offer(&offer).await?;

    // move cyberdeck to another thread to keep it alive
    tokio::spawn(async move {
        while peer.connection_state() != RTCPeerConnectionState::Closed
            && peer.connection_state() != RTCPeerConnectionState::Disconnected
            && peer.connection_state() != RTCPeerConnectionState::Failed
        {
            // keep the connection alive while not in invalid state
            sleep(Duration::from_millis(1000)).await;
        }
        // because we moved cyberdeck ownership into here gets dropped here and will stop all channels
    });

    Ok(answer)
}

// basic handler that responds with a static string
async fn root() -> impl IntoResponse {
    Html(include_str!("index.html"))
}
