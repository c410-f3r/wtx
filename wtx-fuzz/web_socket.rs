//! WebSocket

#![expect(clippy::unwrap_used, reason = "does not matter")]
#![no_main]

use wtx::{
  executor::Runtime,
  rng::{Xorshift64, simple_seed},
  stream::BytesStream,
  web_socket::{Frame, OpCode, WebSocket, WebSocketBuffer},
};

libfuzzer_sys::fuzz_target!(|data: (OpCode, Vec<u8>)| {
  Runtime::new()
    .block_on(async move {
      let Ok(mut ws) = WebSocket::<_, _, _, _, false>::new(
        (),
        false,
        Xorshift64::from(simple_seed()),
        BytesStream::default(),
        WebSocketBuffer::default(),
      ) else {
        return;
      };
      ws.set_max_payload_len(u16::MAX.into());
      let mut frame = Frame::new_fin(data.0, data.1);
      if ws.write_frame(&mut frame).await.is_err() {
        return;
      };
      let _rslt = ws.read_frame().await;
    })
    .unwrap();
});
