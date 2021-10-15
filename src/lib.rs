use anyhow::anyhow;
use anyhow::Result;
use interceptor::registry::Registry;
use std::sync::Arc;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
pub use webrtc::data::data_channel::data_channel_message::DataChannelMessage;
use webrtc::data::data_channel::RTCDataChannel;
use webrtc::peer::configuration::RTCConfiguration;
use webrtc::peer::ice::ice_server::RTCIceServer;
use webrtc::peer::peer_connection::RTCPeerConnection;
use webrtc::peer::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer::sdp::session_description::RTCSessionDescription;
use std::cell::RefCell;

pub type MessageHandler = Box<dyn Fn(DataChannelMessage) + Send + Sync>;

pub struct Cyberdeck {
    peer_connection: Arc<RTCPeerConnection>,
    handle_message: Option<Arc<RefCell<MessageHandler>>>,
}

impl Cyberdeck {
    pub async fn new() -> Result<Cyberdeck> {
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
            handle_message: None,
        });
    }

    pub fn set_message_handler(
        &mut self,
        handle_message: impl Fn(DataChannelMessage) + Send + Sync + 'static,
    ) {
        self.handle_message = Some(Arc::new(RefCell::new(Box::new(handle_message))));
    }

    pub async fn connect(&mut self, offer: String) -> Result<String> {
        // HELP: I copy the atomic reference here if it exists
        let handler:Option<Arc<RefCell<Box<(dyn Fn(DataChannelMessage) + Send + Sync + 'static)>>>> = if let Some(h) = &self.handle_message  {
            Some(h.clone())
        } else {
            None
        };

        self.peer_connection
            .on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
                println!("Peer Connection State has changed: {}", s);
                Box::pin(async {})
            }))
            .await;

        self.peer_connection
            .on_data_channel(Box::new(|d: Arc<RTCDataChannel>| {
                Box::pin(async move {
                    /*d.on_open(Box::new(move || {
                        //todo handle open
                        Box::pin(async {})
                    }))
                    .await;*/

                    d.on_message(Box::new( |msg: DataChannelMessage| {
                        // HELP: Things go crazy here
                        let h = handler;
                        let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
                        println!("Message from DataChannel '{}'", msg_str);
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
