use anyhow::anyhow;
use anyhow::Result;
use async_std::task;
use interceptor::registry::Registry;
use std::sync::Arc;
use std::sync::Mutex;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data::data_channel::data_channel_message::DataChannelMessage;
pub use webrtc::data::data_channel::RTCDataChannel;
use webrtc::peer::configuration::RTCConfiguration;
use webrtc::peer::ice::ice_server::RTCIceServer;
use webrtc::peer::peer_connection::RTCPeerConnection;
use webrtc::peer::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer::sdp::session_description::RTCSessionDescription;

pub struct Connection {
    conn: Arc<RTCDataChannel>,
}

impl Connection {
    pub fn name(&self) -> String {
        self.conn.label().to_string()
    }
    pub fn send_text(&self, msg: &str) -> Result<usize, webrtc::Error> {
        task::block_on(self.conn.send_text(msg.to_string()))
    }
}

pub struct Cyberdeck {
    peer_connection: Arc<RTCPeerConnection>,
    handle_open: Arc<Mutex<Option<Box<dyn Fn(Connection) + Send + Sync + 'static>>>>,
    handle_message:
        Arc<Mutex<Option<Box<dyn Fn(Connection, DataChannelMessage) + Send + Sync + 'static>>>>,
}

impl Cyberdeck {
    pub async fn new(
        handle_open: impl Fn(Connection) + Send + Sync + 'static,
        handle_message: impl Fn(Connection, DataChannelMessage) + Send + Sync + 'static,
    ) -> Result<Cyberdeck> {
        let mut m = MediaEngine::default();
        m.register_default_codecs()?;
        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut m)?;

        let api = APIBuilder::new()
            .with_media_engine(m)
            .with_interceptor_registry(registry)
            .build();

        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };

        let peer_connection = Arc::new(api.new_peer_connection(config).await?);
        return Ok(Cyberdeck {
            peer_connection,
            handle_open: Arc::new(Mutex::new(Some(Box::new(handle_open)))),
            handle_message: Arc::new(Mutex::new(Some(Box::new(handle_message)))),
        });
    }

    pub async fn connect(&mut self, offer: String) -> Result<String> {
        let open_handler = self.handle_open.clone();
        let handler = self.handle_message.clone();

        self.peer_connection
            .on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
                println!("Peer Connection State has changed: {}", s);
                Box::pin(async {})
            }))
            .await;

        self.peer_connection
            .on_data_channel(Box::new(move |d: Arc<RTCDataChannel>| {
                let h2 = handler.clone();
                let o2 = open_handler.clone();

                Box::pin(async move {
                    let d2 = d.clone();
                    let d4 = d.clone();
                    d.on_open(Box::new(move || {
                        let d3 = d2.clone();
                        let o4 = &o2;
                        let o5 = o4.lock().unwrap();
                        if let Some(f) = o5.as_ref() {
                            f(Connection { conn: d3 });
                        }
                        Box::pin(async {})
                    }))
                    .await;

                    d.on_message(Box::new(move |msg: DataChannelMessage| {
                        let d5 = d4.clone();
                        let h4 = &h2;
                        let h5 = h4.lock().unwrap();
                        if let Some(f) = h5.as_ref() {
                            f(Connection { conn: d5 }, msg);
                        }
                        Box::pin(async {})
                    }))
                    .await;
                })
            }))
            .await;

        let desc_data = decode(offer.as_str())?.to_string();
        let offer = serde_json::from_str::<RTCSessionDescription>(&desc_data)?;
        self.peer_connection.set_remote_description(offer).await?;
        let answer = self.peer_connection.create_answer(None).await?;
        let mut gather_complete = self.peer_connection.gathering_complete_promise().await;
        self.peer_connection.set_local_description(answer).await?;
        let _ = gather_complete.recv().await;

        if let Some(local_desc) = self.peer_connection.local_description().await {
            let json_str = serde_json::to_string(&local_desc)?;
            let b64 = encode(&json_str);
            return Ok(b64);
        } else {
            return Err(anyhow!("generate local_description failed!"));
        }
    }

    pub async fn close(&mut self) -> Result<(), webrtc::Error> {
        self.peer_connection.close().await
    }
}

pub fn encode(b: &str) -> String {
    base64::encode(b)
}

pub fn decode(s: &str) -> Result<String> {
    let b = base64::decode(s)?;
    let s = String::from_utf8(b)?;
    Ok(s)
}
