# WTX 

[![CI](https://github.com/c410-f3r/wtx/workflows/CI/badge.svg)](https://github.com/c410-f3r/wtx/actions?query=workflow%3ACI)
[![crates.io](https://img.shields.io/crates/v/wtx.svg)](https://crates.io/crates/wtx)
[![Documentation](https://docs.rs/wtx/badge.svg)](https://docs.rs/wtx)
[![License](https://img.shields.io/badge/license-APACHE2-blue.svg)](./LICENSE)
[![Rustc](https://img.shields.io/badge/rustc-1.71-lightgray")](https://blog.rust-lang.org/2020/03/12/Rust-1.71.html)

A collection of different web transport implementations.

## WebSocket

Provides low and high level abstractions to dispatch frames, as such, it is up to you to implement [Stream](https://docs.rs/wtx/latest/wtx/trait.Stream.html) with any desired logic or use any of the built-in strategies through the selection of features.


```rust
use wtx::{
  Stream,
  rng::Rng,
  web_socket::{
    FrameBufferVec, FrameMutVec, FrameVecMut, compression::NegotiatedCompression, OpCode,
    WebSocketClientOwned
  }
};

pub async fn handle_client_frames(
  fb: &mut FrameBufferVec,
  ws: &mut WebSocketClientOwned<impl NegotiatedCompression, impl Rng, impl Stream>
  ) -> wtx::Result<()> {
  loop {
    let frame = match ws.read_frame(fb).await {
      Err(err) => {
        println!("Error: {err}");
        ws.write_frame(&mut FrameMutVec::new_fin(fb, OpCode::Close, &[])?).await?;
        break;
      }
      Ok(elem) => elem,
    };
    match (frame.op_code(), frame.text_payload()) {
      (_, Some(elem)) => println!("{elem}"),
      (OpCode::Close, _) => break,
      _ => {}
    }
  }
  Ok(())
}
```

See the `examples` directory for more suggestions.

### Autobahn

All the `fuzzingclient`/`fuzzingserver` tests provided by the Autobahn|Testsuite project are passing and the full reports can found at https://c410-f3r.github.io/wtx.

### Compression

The "permessage-deflate" extension, described in [RFC-7692](https://datatracker.ietf.org/doc/html/rfc7692), is the only supported compression format and is backed by the fastest compression library available at the current time, which is "zlib-ng". It also means that a C compiler is required to use such a feature.

### Performance

There are mainly 2 things that impact performance, the chosen runtime and the number of pre-allocated bytes. Specially for servers that have to create a new `WebSocket` instance for each handshake, pre-allocating a high number of bytes for short-lived or low-transfer connections can have a negative impact.

![Benchmark](https://i.imgur.com/Iv2WzJV.jpg)

If you disagree with any of the above numbers, feel free to checkout `wtx-bench` to point any misunderstandings or misconfigurations. A more insightful analysis is available at https://c410-f3r.github.io/thoughts/the-fastest-websocket-implementation/.
