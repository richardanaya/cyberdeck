use anyhow::Result;
use cyberdeck::*;

#[tokio::main]
async fn main() -> Result<()> {
    let offer = must_read_stdin()?;
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

    println!(
        "Type in this code into the other website/terminal app: {}",
        answer
    );
    tokio::signal::ctrl_c().await?;
    peer.close().await?;
    Ok(())
}

pub fn must_read_stdin() -> Result<String> {
    let mut line = String::new();

    std::io::stdin().read_line(&mut line)?;
    line = line.trim().to_owned();
    println!();

    Ok(line)
}
