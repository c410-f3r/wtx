use tokio::{
  io::{AsyncBufReadExt, BufReader},
  net::{TcpListener, TcpStream},
};
use wtx::{
  rng::StdRng,
  web_socket::{
    handshake::{WebSocketAccept, WebSocketAcceptRaw, WebSocketConnect, WebSocketConnectRaw},
    FrameBufferVec, FrameMutVec, OpCode,
  },
  PartitionedBuffer, UriParts,
};

pub(crate) async fn _connect(uri: &str, cb: impl Fn(&str)) -> wtx::Result<()> {
  let uri_parts = UriParts::from(uri);
  let fb = &mut FrameBufferVec::default();
  let pb = &mut <_>::default();
  let (_, mut ws) = WebSocketConnectRaw {
    fb,
    headers_buffer: &mut <_>::default(),
    pb,
    rng: StdRng::default(),
    stream: TcpStream::connect(uri_parts.host).await?,
    uri,
    compression: (),
  }
  .connect()
  .await?;
  let mut buffer = String::new();
  let mut reader = BufReader::new(tokio::io::stdin());
  loop {
    tokio::select! {
      frame_rslt = ws.read_frame(fb) => {
        let frame = frame_rslt?;
        match (frame.op_code(), frame.text_payload()) {
          (_, Some(elem)) => cb(elem),
          (OpCode::Close, _) => break,
          _ => {}
        }
      }
      read_rslt = reader.read_line(&mut buffer) => {
        let _ = read_rslt?;
        ws.write_frame(&mut FrameMutVec::new_fin(fb, OpCode::Text, buffer.as_bytes())?).await?;
      }
    }
  }
  Ok(())
}

pub(crate) async fn _serve(
  uri: &str,
  binary: fn(&[u8]),
  error: fn(wtx::Error),
  str: fn(&str),
) -> wtx::Result<()> {
  let uri_parts = UriParts::from(uri);
  let listener = TcpListener::bind(uri_parts.host).await?;
  loop {
    let (stream, _) = listener.accept().await?;
    let _jh = tokio::spawn(async move {
      let sun = || async move {
        let pb = PartitionedBuffer::default();
        let mut ws = WebSocketAcceptRaw {
          compression: (),
          key_buffer: &mut <_>::default(),
          pb,
          rng: StdRng::default(),
          stream,
        }
        .accept(|_| true)
        .await?;
        let mut fb = FrameBufferVec::default();
        loop {
          let frame = ws.read_frame(&mut fb).await?;
          match (frame.op_code(), frame.text_payload()) {
            (_, Some(elem)) => str(elem),
            (OpCode::Binary, _) => binary(frame.fb().payload()),
            (OpCode::Close, _) => break,
            _ => {}
          }
        }
        wtx::Result::Ok(())
      };
      if let Err(err) = sun().await {
        error(err);
      }
    });
  }
}
