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

async fn connect(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(offer): Json<String>,
) -> impl IntoResponse {
    match start_connection(offer).await {
        Ok(answer) => Ok(Json(answer)),
        Err(_) => Err("failed to connect"),
    }
}

async fn start_connection(offer: String) -> Result<String> {
    let mut cd = Cyberdeck::new(|e| async move {
        match e {
            CyberdeckEvent::DataChannelMessage(c, m) => {
                println!("Recieved a message from channel {}!", c.name());
                let msg_str = String::from_utf8(m.data.to_vec()).unwrap();
                println!("Message from DataChannel '{}': {}", c.name(), msg_str);
            }
            CyberdeckEvent::DataChannelStateChange(c) => {
                if c.state() == RTCDataChannelState::Open {
                    println!("DataChannel '{}' opened", c.name());
                    c.send_text("Connected to client!").await.unwrap();
                } else if c.state() == RTCDataChannelState::Closed {
                    println!("DataChannel '{}' closed", c.name());
                }
            }
            CyberdeckEvent::PeerConnectionStateChange(s) => {
                println!("Peer connection state: {} ", s)
            }
        }
    })
    .await?;
    let answer = cd.receive_offer(&offer).await?;
    tokio::spawn(async move {
        while cd.connection_state() != RTCPeerConnectionState::Closed
            && cd.connection_state() != RTCPeerConnectionState::Disconnected
            && cd.connection_state() != RTCPeerConnectionState::Failed
        {
            // keep the connection alive while not in invalid state
            sleep(Duration::from_millis(1000)).await;
        }
    });
    Ok(answer)
}

// basic handler that responds with a static string
async fn root() -> impl IntoResponse {
    Html(include_str!("index.html"))
}
