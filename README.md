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
let answer = cd.receive_offer(offer).await?;
```

You can try out this code by going to https://jsfiddle.net/ndgvLuyc/

1. Copy the code from "Browser base64 Session Description"
2. Open up a terminal and type in `echo <code you copied from above> | cargo run --example receiver`
3. Copy the response code in the terminal, and past it into "Rust base64 Session Description"
4. Hit connect and send messages


Limitations:
* Right now this only seems to work app to website. Not quite sure how to handle ICE servers yet. If you want to test this out though:

1. `cargo run --example offer`
2. copy the code
3. `echo <code you copied from above> | cargo run --example receiver`
4. copy the code the receiver app gives you
5. paste the code into the terminal of the offer app and press enter

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
