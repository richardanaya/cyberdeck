use anyhow::Result;
use cyberdeck::*;

#[tokio::main]
async fn main() -> Result<()> {
    let offer = must_read_stdin()?;
    let mut cd = Cyberdeck::new( |c, msg| async move {
        if let Some(m) = msg {
            println!("Recieved a message from channel {}!", c.name());
            let msg_str = String::from_utf8(m.data.to_vec()).unwrap();
            println!("Message from DataChannel '{}': {}", c.name(), msg_str);
        } else if c.state() == RTCDataChannelState::Open {
            println!("DataChannel '{}' opened", c.name());
            c.send_text("Connected to client!").await.unwrap();
        } else if c.state() == RTCDataChannelState::Closed {
            println!("DataChannel '{}' closed", c.name());
        }
    })
    .await?;
    let answer = cd.set_offer(offer).await?;

    println!("Type in this code into the website: {}", answer);
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
