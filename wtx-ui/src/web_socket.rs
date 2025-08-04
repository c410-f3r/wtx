use tokio::{
  io::{AsyncBufReadExt, BufReader},
  net::{TcpListener, TcpStream},
};
use wtx::{
  collection::Vector,
  misc::UriRef,
  web_socket::{Frame, OpCode, WebSocketAcceptor, WebSocketConnector, WebSocketReadMode},
};

pub(crate) async fn connect(uri: &str, cb: impl Fn(&str)) -> wtx::Result<()> {
  let uri = UriRef::new(uri);
  let mut ws = WebSocketConnector::default()
    .connect(TcpStream::connect(uri.hostname_with_implied_port()).await?, &uri)
    .await?;
  let mut read_frame_buffer = Vector::new();
  let mut stdin_buffer = Vec::new();
  let mut buf_reader = BufReader::new(tokio::io::stdin());
  loop {
    tokio::select! {
      frame_rslt = ws.read_frame(&mut read_frame_buffer, WebSocketReadMode::Adaptive) => {
        let frame = frame_rslt?;
        match (frame.op_code(), frame.text_payload()) {
          (_, Some(elem)) => cb(elem),
          (OpCode::Close, _) => break,
          _ => {}
        }
      }
      read_rslt = buf_reader.read_until(b'\n', &mut stdin_buffer) => {
        let _ = read_rslt?;
        ws.write_frame(&mut Frame::new_fin(OpCode::Text, &mut stdin_buffer)).await?;
      }
    }
  }
  Ok(())
}

pub(crate) async fn serve(
  uri: &str,
  binary: fn(&[u8]),
  error: fn(wtx::Error),
  str: fn(&str),
) -> wtx::Result<()> {
  let uri = UriRef::new(uri);
  let listener = TcpListener::bind(uri.hostname_with_implied_port()).await?;
  loop {
    let (stream, _) = listener.accept().await?;
    let _jh = tokio::spawn(async move {
      let fun = async move {
        let mut buffer = Vector::new();
        let mut ws = WebSocketAcceptor::default().accept(stream).await?;
        loop {
          let frame = ws.read_frame(&mut buffer, WebSocketReadMode::Adaptive).await?;
          match (frame.op_code(), frame.text_payload()) {
            (_, Some(elem)) => str(elem),
            (OpCode::Binary, _) => binary(frame.payload()),
            (OpCode::Close, _) => break,
            _ => {}
          }
        }
        wtx::Result::Ok(())
      };
      if let Err(err) = fun.await {
        error(err);
      }
    });
  }
}
