# WebSocket

Implementation of [RFC6455](https://datatracker.ietf.org/doc/html/rfc6455) and [RFC7692](https://datatracker.ietf.org/doc/html/rfc7692).

Activation feature is called `web-socket`.

![WebSocket Benchmark](https://i.imgur.com/Iv2WzJV.jpg)

```rust,edition2021
extern crate wtx;

use wtx::{
  misc::Stream,
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
        eprintln!("Error: {err}");
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

## Autobahn Reports

1. <a href="https://c410-f3r.github.io/wtx-site/static/fuzzingclient/index.html" target="_blank">fuzzingclient</a>
2. <a href="https://c410-f3r.github.io/wtx-site/static/fuzzingserver/index.html" target="_blank">fuzzingserver</a>

## Compression

The "permessage-deflate" extension is the only supported compression format and is backed by the `zlib-rs` project that performs as well as `zlib-ng`.

To get the most performance possible, try compiling your program with `RUSTFLAGS='-C target-cpu=native'` to allow `zlib-rs` to use more efficient SIMD instructions.