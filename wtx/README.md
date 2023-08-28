# WTX 

[![CI](https://github.com/c410-f3r/wtx/workflows/CI/badge.svg)](https://github.com/c410-f3r/wtx/actions?query=workflow%3ACI)
[![crates.io](https://img.shields.io/crates/v/wtx.svg)](https://crates.io/crates/wtx)
[![Documentation](https://docs.rs/wtx/badge.svg)](https://docs.rs/wtx)
[![License](https://img.shields.io/badge/license-APACHE2-blue.svg)](./LICENSE)
[![Rustc](https://img.shields.io/badge/rustc-1.71-lightgray")](https://blog.rust-lang.org/2020/03/12/Rust-1.71.html)

Different web transport implementations.

## WebSocket

Provides low and high level abstractions to dispatch frames, as such, it is up to you to implement [Stream](https://docs.rs/wtx/latest/wtx/trait.Stream.html) with any desired logic or use any of the built-in strategies through the selection of features.

[fastwebsockets](https://github.com/denoland/fastwebsockets) served as an initial inspiration for the skeleton of this implementation so thanks to the authors.

```rust
use wtx::{
  Stream, web_socket::{FrameBufferVec, FrameMutVec, FrameVecMut, OpCode, WebSocketClientOwned}
};

pub async fn handle_client_frames(
  fb: &mut FrameBufferVec,
  ws: &mut WebSocketClientOwned<impl Stream>
  ) -> wtx::Result<()> {
  loop {
    let frame = match ws.read_msg(fb).await {
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

### Performance

There are mainly 2 things that impact performance, the chosen runtime and the number of pre-allocated bytes. Specially for servers that have to create a new `WebSocket` instance for each handshake, pre-allocating a high number of bytes for short-lived or low-transfer connections can have a negative impact.

![Benchmark](https://i.imgur.com/ZZU3Hay.jpeg)