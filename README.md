# Cyberdeck
A library for easily creating WebRTC data channel connections in Rust.

```rust
let mut cd = Cyberdeck::new(|c, msg| {
    if let Some(m) = msg {
        println!("Recieved a message from channel {}!", c.name());
        let msg_str = String::from_utf8(m.data.to_vec()).unwrap();
        println!("Message from DataChannel '{}': {}", c.name(), msg_str);
    } else if c.state() == RTCDataChannelState::Open {
        println!("DataChannel '{}' opened", c.name());
        c.send_text("Connected to client!").unwrap();
    } else if c.state() == RTCDataChannelState::Closed {
        println!("DataChannel '{}' closed", c.name());
    }
})
.await?;
let answer = cd.connect(offer).await?;
```

You can try out this code by going to https://jsfiddle.net/ndgvLuyc/

1. Copy the code from "Browser base64 Session Description"
2. Open up a terminal and type in `echo <code you copied from above> | cargo run --example simple`
3. Copy the response code in the terminal, and past it into "Rust base64 Session Description"
4. Hit connect and send messages
