# WebSocket

Provides low and high level abstractions to dispatch frames, as such, it is up to you to implement [Stream](https://docs.rs/wtx/latest/wtx/trait.Stream.html) with any desired logic or use any of the built-in strategies through the selection of features.

Activation feature is called `web-socket`.

![WebSocket Benchmark](https://i.imgur.com/Iv2WzJV.jpg)

```rust,edition2021
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

See the `examples` directory for more suggestions.

## Autobahn Reports

1. <a href="https://c410-f3r.github.io/wtx-site/static/fuzzingclient/index.html" target="_blank">fuzzingclient</a>
2. <a href="https://c410-f3r.github.io/wtx-site/static/fuzzingserver/index.html" target="_blank">fuzzingserver</a>

## Compression

The "permessage-deflate" extension, described in [RFC-7692](https://datatracker.ietf.org/doc/html/rfc7692), is the only supported compression format and is backed by the fastest compression library available at the current time, which is "zlib-ng". It also means that a C compiler is required to use such a feature.
