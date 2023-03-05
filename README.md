<a href="https://docs.rs/cyberdeck"><img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square" alt="docs.rs docs" /></a>

# Cyberdeck
A library for easily creating WebRTC data channel connections in Rust.

```toml
[dependencies]
cyberdeck = "0.0.12"
```

```rust
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
```

You can try out this code by going to https://jsfiddle.net/ndgvLuyc/

1. Copy the code from "Browser base64 Session Description"
2. Open up a terminal and type in `echo <code you copied from above> | cargo run --example receiver`
3. Copy the response code in the terminal, and past it into "Rust base64 Session Description"
4. Hit connect and send messages

# Signaling server

WebRTC works in it's most basic form by having the client and server exchange strings that represent their networking information.  A signaling server is just some API that you exchange that information through. You can see a simple signaling server implemented with a single POST http handler here in this example [here](https://github.com/richardanaya/cyberdeck/blob/master/examples/signaling_server.rs).

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
