use wtx::{
  misc::Stream,
  rng::StaticRng,
  web_socket::{
    compression::NegotiatedCompression,
    handshake::{WebSocketAccept, WebSocketAcceptRaw},
    Compression, FrameBufferVec, OpCode, WebSocketBuffer, WebSocketServerMut,
  },
};

pub(crate) async fn _accept_conn_and_echo_frames<C, S>(
  compression: C,
  fb: &mut FrameBufferVec,
  stream: S,
  wsb: &mut WebSocketBuffer,
) -> wtx::Result<()>
where
  C: Compression<false>,
  S: Stream,
{
  let mut ws = WebSocketAcceptRaw { compression, rng: StaticRng::default(), stream, wsb }
    .accept(|_| true)
    .await?;
  _handle_frames(fb, &mut ws).await?;
  Ok(())
}

pub(crate) async fn _handle_frames<NC, S>(
  fb: &mut FrameBufferVec,
  ws: &mut WebSocketServerMut<'_, NC, StaticRng, S>,
) -> wtx::Result<()>
where
  NC: NegotiatedCompression,
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
