use anyhow::anyhow;
use anyhow::Result;
use async_std::task;
pub use bytes::Bytes;
use interceptor::registry::Registry;
use std::mem;
use std::sync::Arc;
use std::sync::Mutex;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data::data_channel::data_channel_message::DataChannelMessage;
pub use webrtc::data::data_channel::data_channel_state::RTCDataChannelState;
pub use webrtc::data::data_channel::RTCDataChannel;
use webrtc::peer::configuration::RTCConfiguration;
use webrtc::peer::ice::ice_server::RTCIceServer;
use webrtc::peer::peer_connection::RTCPeerConnection;
use webrtc::peer::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer::sdp::session_description::RTCSessionDescription;
use tokio::sync::mpsc;

pub struct Configuration {
    stun_or_turn_urls: Vec<String>,
}

pub struct Connection {
    conn: Arc<RTCDataChannel>,
}

impl Connection {
    pub fn name(&self) -> &str {
        self.conn.label()
    }

    pub fn state(&self) -> RTCDataChannelState {
        self.conn.ready_state()
    }

    pub fn send(&self, data: &Bytes) -> Result<usize, webrtc::Error> {
        task::block_on(self.conn.send(data))
    }

    pub fn send_text(&self, msg: &str) -> Result<usize, webrtc::Error> {
        task::block_on(self.conn.send_text(msg.to_string()))
    }
}

pub struct Cyberdeck {
    peer_connection: Arc<RTCPeerConnection>,
    handle_message: Arc<
        Mutex<Option<Box<dyn Fn(Connection, Option<DataChannelMessage>) + Send + Sync + 'static>>>,
    >,
}

impl Cyberdeck {
    pub async fn new(
        handle_message: impl Fn(Connection, Option<DataChannelMessage>) + Send + Sync + 'static,
    ) -> Result<Cyberdeck> {
        Cyberdeck::new_with_configuration(
            handle_message,
            Configuration {
                stun_or_turn_urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            },
        )
        .await
    }

    pub async fn new_with_configuration(
        handle_message: impl Fn(Connection, Option<DataChannelMessage>) + Send + Sync + 'static,
        mut config: Configuration,
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
                urls: mem::take(&mut config.stun_or_turn_urls),
                ..Default::default()
            }],
            ..Default::default()
        };

        let peer_connection = Arc::new(api.new_peer_connection(config).await?);
        let mut c = Cyberdeck {
            peer_connection,
            handle_message: Arc::new(Mutex::new(Some(Box::new(handle_message)))),
        };
        c.setup().await;
        return Ok(c);
    }

    async fn setup(&mut self) {
        let handler = self.handle_message.clone();
        let (tx1, mut rx) = mpsc::unbounded_channel();
        let tx2 = tx1.clone();
        let tx3 = tx1.clone();

        self.peer_connection
            .on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
                println!("Peer Connection State has changed: {}", s);
                Box::pin(async {})
            }))
            .await;

        self.peer_connection
            .on_data_channel(Box::new(move |d: Arc<RTCDataChannel>| {
                let handler_clone1 = handler.clone();
                let handler_clone2 = handler.clone();
                let handler_clone3 = handler.clone();

                Box::pin(async move {
                    let data_cannel_clone1 = d.clone();
                    let data_cannel_clone2 = d.clone();
                    let data_cannel_clone3 = d.clone();
                    d.on_open(Box::new(move || {
                        if let Some(f) = (&handler_clone1).lock().unwrap().as_ref() {
                            f(
                                Connection {
                                    conn: data_cannel_clone1.clone(),
                                },
                                None,
                            );
                        }
                        Box::pin(async {})
                    }))
                    .await;

                    d.on_close(Box::new(move || {
                        if let Some(f) = (&handler_clone2).lock().unwrap().as_ref() {
                            f(
                                Connection {
                                    conn: data_cannel_clone2.clone(),
                                },
                                None,
                            );
                        }
                        Box::pin(async {})
                    }))
                    .await;

                    d.on_message(Box::new(move |msg: DataChannelMessage| {
                        if let Some(f) = (&handler_clone3).lock().unwrap().as_ref() {
                            f(
                                Connection {
                                    conn: data_cannel_clone3.clone(),
                                },
                                Some(msg),
                            );
                        }
                        Box::pin(async {})
                    }))
                    .await;
                })
            }))
            .await;
    }

    pub async fn set_offer(&mut self, offer: String) -> Result<String> {
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
