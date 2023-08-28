//! Parse

#![allow(
  // Does not matter
  clippy::unwrap_used
)]
#![no_main]

use tokio::runtime::Handle;
use wtx::{
    web_socket::{FrameBufferVec, FrameVecMut, OpCode, WebSocketServer},
    BytesStream, ReadBuffer,
};

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let mut ws = WebSocketServer::new(ReadBuffer::default(), BytesStream::default());
    ws.set_max_payload_len(u16::MAX.into());
    let fb = &mut FrameBufferVec::default();
    Handle::current().block_on(async move {
        ws.write_frame(&mut FrameVecMut::new_fin(fb.into(), OpCode::Text, data).unwrap())
            .await
            .unwrap();
        let _frame = ws.read_frame(fb).await.unwrap();
    });
});
