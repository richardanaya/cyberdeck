# Cyberdeck
A library for easily creating WebRTC data channel connections in Rust.

```rust
let mut cd = Cyberdeck::new(
    |channel| {
        println!("Data channel {} was connected!", channel.name());
        c.send_text("connected!").expect("could not send message");
    },
    |channel, msg| {
        println!("Recieved a message from channel {}!", channel.name());
        let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
        println!("Message from DataChannel '{}'", msg_str);
    },
)
.await?;
let answer = cd.connect(offer).await?;
```

You can try out this code by going to https://jsfiddle.net/ndgvLuyc/

1. Copy the code from "Browser base64 Session Description"
2. Open up a terminal and type in `echo <code you copied from above> | cargo run --example simple`
3. Copy the response code in the terminal, and past it into "Rust base64 Session Description"
4. Hit connect and send messages
