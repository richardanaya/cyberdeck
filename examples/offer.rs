use anyhow::Result;
use cyberdeck::*;

#[tokio::main]
async fn main() -> Result<()> {
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

    let offer = cd.create_offer().await?;

    println!(
        "Type in this code into the other website/terminal app: {}",
        offer
    );

    println!("Type in this code from the other website/terminal app.");

    let other_offer = must_read_stdin()?;

    cd.receive_offer(other_offer).await?;

    tokio::signal::ctrl_c().await?;
    cd.close().await?;
    Ok(())
}

pub fn must_read_stdin() -> Result<String> {
    let mut line = String::new();

    std::io::stdin().read_line(&mut line)?;
    line = line.trim().to_owned();
    println!();

    Ok(line)
}
