<a href="https://docs.rs/cyberdeck"><img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square" alt="docs.rs docs" /></a>

# Cyberdeck
A library for easily creating WebRTC data channel connections in Rust.

```toml
cargo add cyberdeck
```

```rust
let mut peer = Peer::new(|peer_id, e| async move {
    match e {
        PeerEvent::DataChannelMessage(c, m) => {
            println!(
                "{}::Recieved a message from channel {} with id {}!",
                peer_id,
                c.label(),
                c.id()
            );
            let msg_str = String::from_utf8(m.data.to_vec()).unwrap();
            println!(
                "{}::Message from DataChannel '{}': {}",
                peer_id,
                c.label(),
                msg_str
            );
        }
        PeerEvent::DataChannelStateChange(c) => {
            if c.ready_state() == RTCDataChannelState::Open {
                println!("{}::DataChannel '{}'", peer_id, c.label());
                c.send_text("Connected to client!".to_string())
                    .await
                    .unwrap();
            } else if c.ready_state() == RTCDataChannelState::Closed {
                println!("{}::DataChannel '{}'", peer_id, c.label());
            }
        }
        PeerEvent::PeerConnectionStateChange(s) => {
            println!("{}::Peer connection state: {} ", peer_id, s)
        }
    }
})
.await?;
let answer = peer.receive_offer(&offer).await?;
```

# Signaling server

WebRTC works in it's most basic form by having the client and server exchange strings that represent their networking information.  A signaling server is just some API that you exchange that information through. You can see a simple signaling server implemented with a single POST http handler here in this example [here](https://github.com/richardanaya/cyberdeck/blob/master/examples/signaling_server.rs).

```bash
cargo run --example signaling_server
```

# Art

![Cyberpunk crab](https://user-images.githubusercontent.com/294042/222991163-9ef095eb-98da-419f-8f06-b1ea1d51f34d.png)

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
