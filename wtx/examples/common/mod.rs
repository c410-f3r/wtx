use std::borrow::BorrowMut;
use wtx::{
  rng::StdRng,
  web_socket::{
    compression::NegotiatedCompression, handshake::WebSocketAcceptRaw, Compression, FrameBufferVec,
    OpCode, WebSocketServer,
  },
  PartitionedBuffer, Stream,
};

pub(crate) async fn _accept_conn_and_echo_frames<C, PB, S>(
  compression: C,
  fb: &mut FrameBufferVec,
  pb: PB,
  stream: S,
) -> wtx::Result<()>
where
  C: Compression<false>,
  PB: BorrowMut<PartitionedBuffer>,
  S: Stream,
{
  let (_, mut ws) = WebSocketServer::accept(WebSocketAcceptRaw {
    compression,
    headers_buffer: &mut <_>::default(),
    key_buffer: &mut <_>::default(),
    pb,
    rng: <_>::default(),
    stream,
  })
  .await?;
  _handle_frames(fb, &mut ws).await?;
  Ok(())
}

pub(crate) async fn _handle_frames<NC, PB, S>(
  fb: &mut FrameBufferVec,
  ws: &mut WebSocketServer<NC, PB, StdRng, S>,
) -> wtx::Result<()>
where
  NC: NegotiatedCompression,
  PB: BorrowMut<PartitionedBuffer>,
  S: Stream,
{
  loop {
    let mut frame = ws.read_frame(fb).await?;
    match frame.op_code() {
      OpCode::Binary | OpCode::Text => {
        ws.write_frame(&mut frame).await?;
      }
      OpCode::Close => break,
      _ => {}
    }
  }
  Ok(())
}

pub(crate) fn _host_from_args() -> String {
  std::env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8080".to_owned())
}

pub(crate) fn _uri_from_args() -> String {
  std::env::args().nth(1).unwrap_or_else(|| "http://127.0.0.1:8080".to_owned())
}
