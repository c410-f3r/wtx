//! WebSocket

#![expect(clippy::unwrap_used, reason = "does not matter")]
#![no_main]

use wtx::{
  collection::Vector,
  executor::StdRuntime,
  rng::{SeedableRng, Xorshift64},
  stream::BytesStream,
  tls::{TlsModePlainText, TlsStream},
  web_socket::{Frame, OpCode, WebSocket, WebSocketBuffer, WebSocketPayloadOrigin},
};

libfuzzer_sys::fuzz_target!(|data: (OpCode, Vec<u8>)| {
  StdRuntime::new().block_on(async move {
    let mut ws = WebSocket::<_, _, _, _, false>::new(
      (),
      false,
      Xorshift64::from_simple_seed().unwrap(),
      TlsStream::new(BytesStream::default(), TlsModePlainText),
      WebSocketBuffer::default(),
    );
    ws.set_max_payload_len(u16::MAX.into());
    let mut frame = Frame::new_fin(data.0, data.1).unwrap();
    if ws.write_frame(&mut frame).await.is_err() {
      return;
    };
    let _rslt = ws.read_frame(&mut Vector::new(), WebSocketPayloadOrigin::Adaptive).await;
  });
});
