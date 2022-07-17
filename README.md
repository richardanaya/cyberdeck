<a href="https://docs.rs/cyberdeck"><img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square" alt="docs.rs docs" /></a>

# Cyberdeck
A library for easily creating WebRTC data channel connections in Rust.

```toml
[dependencies]
cyberdeck = "0.0.12"
```

```rust
let mut cd = Cyberdeck::new(|e| async move {
    match e {
        CyberdeckEvent::DataChannelMessage(channel, msg) => {
            println!("Recieved a message from channel {}!", channel.name());
            let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
            println!("Message from DataChannel '{}': {}", channel.name(), msg_str);
        }
        CyberdeckEvent::DataChannelStateChange(channel) => {
            if channel.state() == RTCDataChannelState::Open {
                println!("DataChannel '{}' opened", channel.name());
                channel.send_text("Connected to client!").await.unwrap();
            } else if channel.state() == RTCDataChannelState::Closed {
                println!("DataChannel '{}' closed", channel.name());
            }
        }
        CyberdeckEvent::PeerConnectionStateChange(state) => {
            println!("Peer connection state: {} ", state)
        }
    }
})
.await?;
let answer = cd.receive_offer(offer).await?;
```

You can try out this code by going to https://jsfiddle.net/ndgvLuyc/

1. Copy the code from "Browser base64 Session Description"
2. Open up a terminal and type in `echo <code you copied from above> | cargo run --example receiver`
3. Copy the response code in the terminal, and past it into "Rust base64 Session Description"
4. Hit connect and send messages

# Signaling server

WebRTC works in it's most basic form by having the client and server exchange strings that represent their networking information.  You can see a simple signaling server example [here](https://github.com/richardanaya/cyberdeck/blob/master/examples/signaling_server.rs).

```bash
cargo run --example signaling_server
```

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `cyberdeck` by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
