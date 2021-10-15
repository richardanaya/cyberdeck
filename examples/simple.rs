use anyhow::Result;
use cyberdeck::*;

#[tokio::main]
async fn main() -> Result<()> {
    let offer = must_read_stdin()?;
    let mut cd = Cyberdeck::new(
        |c| {
            println!("Data channel {} was connected!", c.name());
            c.send_text("connected!").expect("could not send message");
        },
        |c, msg| {
            println!("Recieved a message from channel {}!", c.name());
            let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
            println!("Message from DataChannel '{}'", msg_str);
        },
    )
    .await?;
    let answer = cd.connect(offer).await?;

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
