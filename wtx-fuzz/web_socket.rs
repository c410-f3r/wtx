//! WebSocket

#![allow(
    // Does not matter
    clippy::unwrap_used
  )]
#![no_main]

use tokio::runtime::Builder;
use wtx::{
  misc::BytesStream,
  rng::StaticRng,
  web_socket::{FrameBufferVec, FrameMutVec, OpCode, WebSocketBuffer, WebSocketServerOwned},
};

libfuzzer_sys::fuzz_target!(|data: (OpCode, &[u8])| {
  Builder::new_current_thread().enable_all().build().unwrap().block_on(async move {
    let mut ws = WebSocketServerOwned::new(
      (),
      StaticRng::default(),
      BytesStream::default(),
      WebSocketBuffer::default(),
    );
    ws.set_max_payload_len(u16::MAX.into());
    let fb = &mut FrameBufferVec::default();
    let Ok(mut frame) = FrameMutVec::new_fin(fb, data.0, data.1) else {
      return;
    };
    if ws.write_frame(&mut frame).await.is_err() {
      return;
    };
    let _rslt = ws.read_frame(fb).await;
  });
});
