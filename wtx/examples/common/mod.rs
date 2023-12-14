use wtx::{
  misc::{AsyncBounds, Stream},
  rng::StaticRng,
  web_socket::{
    compression::NegotiatedCompression,
    handshake::{WebSocketAccept, WebSocketAcceptRaw},
    Compression, FrameBufferVec, OpCode, WebSocketBuffer, WebSocketServerOwned,
  },
};

pub(crate) async fn _accept_conn_and_echo_frames<C, S>(
  compression: C,
  fb: &mut FrameBufferVec,
  stream: S,
) where
  C: AsyncBounds + Compression<false>,
  C::NegotiatedCompression: AsyncBounds,
  S: AsyncBounds + Stream,
{
  let mut ws = WebSocketAcceptRaw {
    compression,
    rng: <_>::default(),
    stream,
    wsb: WebSocketBuffer::default(),
  }
  .accept(|_| true)
  .await
  .unwrap();
  _handle_frames(fb, &mut ws).await;
}

pub(crate) async fn _handle_frames<NC, S>(
  fb: &mut FrameBufferVec,
  ws: &mut WebSocketServerOwned<NC, StaticRng, S>,
) where
  NC: NegotiatedCompression,
  S: Stream,
{
  loop {
    let mut frame = ws.read_frame(fb).await.unwrap();
    match frame.op_code() {
      OpCode::Binary | OpCode::Text => {
        ws.write_frame(&mut frame).await.unwrap();
      }
      OpCode::Close => break,
      _ => {}
    }
  }
}

pub(crate) fn _host_from_args() -> String {
  std::env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8080".to_owned())
}

pub(crate) fn _uri_from_args() -> String {
  std::env::args().nth(1).unwrap_or_else(|| "http://127.0.0.1:8080".to_owned())
}
