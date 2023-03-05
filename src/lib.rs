use anyhow::anyhow;
use anyhow::Result;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
pub use bytes::Bytes;
use std::future::Future;
use std::mem;
use std::sync::Arc;
use tokio::sync::mpsc;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
pub use webrtc::data_channel::data_channel_init::RTCDataChannelInit;
pub use webrtc::data_channel::data_channel_message::DataChannelMessage;
pub use webrtc::data_channel::data_channel_state::RTCDataChannelState;
pub use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
pub use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;

pub struct Configuration {
    stun_or_turn_urls: Vec<String>,
}

pub type DataChannel = Arc<RTCDataChannel>;

pub struct Peer {
    pub peer_id: u128,
    peer_connection: Arc<RTCPeerConnection>,
    abort: mpsc::UnboundedSender<()>,
}

pub enum PeerEvent {
    PeerConnectionStateChange(RTCPeerConnectionState),
    DataChannelStateChange(DataChannel),
    DataChannelMessage(DataChannel, DataChannelMessage),
}

impl Peer {
    pub async fn new<T>(
        handle_message: impl Fn(u128, PeerEvent) -> T + Send + Sync + 'static,
    ) -> Result<Peer>
    where
        T: Future<Output = ()> + Send + Sync,
    {
        Peer::new_with_configuration(
            handle_message,
            Configuration {
                stun_or_turn_urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            },
        )
        .await
    }

    pub async fn new_with_configuration<T>(
        handle_message: impl Fn(u128, PeerEvent) -> T + Send + Sync + 'static,
        mut config: Configuration,
    ) -> Result<Peer>
    where
        T: Future<Output = ()> + Send + Sync,
    {
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

        let (tx, mut msg_rx) = mpsc::unbounded_channel::<(u128, PeerEvent)>();
        let tx_clone = tx.clone();
        let (abort_tx, mut abort_rx) = mpsc::unbounded_channel::<()>();
        let abort_tx_clone = abort_tx.clone();

        let peer_id = Peer::random_peer_id();
        let c = Peer {
            peer_id,
            peer_connection,
            abort: abort_tx,
        };

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    val = msg_rx.recv() => {
                        if let Some(v) = val {
                            handle_message(v.0,v.1).await;
                        }
                    }
                    _ = abort_rx.recv() => {
                        break;
                    }
                };
            }
        });

        c.peer_connection.on_peer_connection_state_change(Box::new(
            move |s: RTCPeerConnectionState| {
                match tx_clone.send((peer_id, PeerEvent::PeerConnectionStateChange(s))) {
                    Ok(_) => (),
                    Err(error) => {
                        panic!("Error sending mpsc message: {:?}", error.to_string())
                    }
                };
                if s == RTCPeerConnectionState::Failed {
                    match abort_tx_clone.send(()) {
                        Ok(_) => (),
                        Err(error) => {
                            panic!("Error sending mpsc message: {:?}", error.to_string())
                        }
                    };
                }
                Box::pin(async {})
            },
        ));

        c.peer_connection
            .on_data_channel(Box::new(move |d: Arc<RTCDataChannel>| {
                let tx1 = tx.clone();
                let tx2 = tx.clone();
                let tx3 = tx.clone();

                Box::pin(async move {
                    let data_cannel_clone1 = d.clone();
                    let data_cannel_clone2 = d.clone();
                    let data_cannel_clone3 = d.clone();
                    d.on_open(Box::new(move || {
                        match tx1.send((
                            peer_id,
                            PeerEvent::DataChannelStateChange(data_cannel_clone1.clone()),
                        )) {
                            Ok(_) => (),
                            Err(error) => {
                                panic!("Error sending mpsc message: {:?}", error.to_string())
                            }
                        };
                        Box::pin(async {})
                    }));

                    d.on_close(Box::new(move || {
                        match tx2.send((
                            peer_id,
                            PeerEvent::DataChannelStateChange(data_cannel_clone2.clone()),
                        )) {
                            Ok(_) => (),
                            Err(error) => {
                                panic!("Error sending mpsc message: {:?}", error.to_string())
                            }
                        };
                        Box::pin(async {})
                    }));

                    d.on_message(Box::new(move |msg: DataChannelMessage| {
                        match tx3.send((
                            peer_id,
                            PeerEvent::DataChannelMessage(data_cannel_clone3.clone(), msg),
                        )) {
                            Ok(_) => (),
                            Err(error) => {
                                panic!("Error sending mpsc message: {:?}", error.to_string())
                            }
                        };
                        Box::pin(async {})
                    }));
                })
            }));

        Ok(c)
    }

    pub async fn create_offer(&mut self) -> Result<String> {
        let offer = self.peer_connection.create_offer(None).await?;

        // Sets the LocalDescription, and starts our UDP listeners
        // Note: this will start the gathering of ICE candidates
        let mut gather_complete = self.peer_connection.gathering_complete_promise().await;
        self.peer_connection.set_local_description(offer).await?;
        let _ = gather_complete.recv().await;

        if let Some(local_desc) = self.peer_connection.local_description().await {
            let json_str = serde_json::to_string(&local_desc)?;
            let b64 = encode(&json_str);
            Ok(b64)
        } else {
            Err(anyhow!("generate local_description failed!"))
        }
    }

    pub async fn receive_offer(&mut self, offer: &str) -> Result<String> {
        let desc_data = decode(offer)?.to_string();
        let offer = serde_json::from_str::<RTCSessionDescription>(&desc_data)?;
        self.peer_connection.set_remote_description(offer).await?;
        let answer = self.peer_connection.create_answer(None).await?;
        let mut gather_complete = self.peer_connection.gathering_complete_promise().await;
        self.peer_connection.set_local_description(answer).await?;
        let _ = gather_complete.recv().await;

        if let Some(local_desc) = self.peer_connection.local_description().await {
            let json_str = serde_json::to_string(&local_desc)?;
            let b64 = encode(&json_str);
            Ok(b64)
        } else {
            Err(anyhow!("generate local_description failed!"))
        }
    }

    pub async fn create_channel(&mut self, name: &str) -> Result<(), webrtc::Error> {
        match self.peer_connection.create_data_channel(name, None).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub async fn create_channel_with_configuration(
        &mut self,
        name: &str,
        config: RTCDataChannelInit,
    ) -> Result<(), webrtc::Error> {
        match self
            .peer_connection
            .create_data_channel(name, Some(config))
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub async fn close(&mut self) -> Result<(), webrtc::Error> {
        self.abort.send(())?;
        self.peer_connection.close().await
    }

    pub fn connection_state(&self) -> RTCPeerConnectionState {
        self.peer_connection.connection_state()
    }

    pub fn random_peer_id() -> u128 {
        rand::random()
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        self.abort.send(()).expect("could not stop task on drop");
    }
}

fn encode(b: &str) -> String {
    STANDARD.encode(b)
}

fn decode(s: &str) -> Result<String> {
    let b = STANDARD.decode(s)?;
    let s = String::from_utf8(b)?;
    Ok(s)
}
