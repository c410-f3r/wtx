use tokio::{
  io::{AsyncBufReadExt, BufReader},
  net::{TcpListener, TcpStream},
};
use wtx::{
  misc::{simple_seed, UriRef, Xorshift64},
  web_socket::{Frame, OpCode, WebSocketBuffer, WebSocketClient, WebSocketServer},
};

pub(crate) async fn connect(uri: &str, cb: impl Fn(&str)) -> wtx::Result<()> {
  let uri = UriRef::new(uri);
  let wsb = &mut WebSocketBuffer::default();
  let mut ws = WebSocketClient::connect(
    (),
    [],
    Xorshift64::from(simple_seed()),
    TcpStream::connect(uri.hostname_with_implied_port()).await?,
    &uri,
    wsb,
    |_| wtx::Result::Ok(()),
  )
  .await?;
  let mut buffer = Vec::new();
  let mut reader = BufReader::new(tokio::io::stdin());
  loop {
    tokio::select! {
      frame_rslt = ws.read_frame() => {
        let frame = frame_rslt?;
        match (frame.op_code(), frame.text_payload()) {
          (_, Some(elem)) => cb(elem),
          (OpCode::Close, _) => break,
          _ => {}
        }
      }
      read_rslt = reader.read_until(b'\n', &mut buffer) => {
        let _ = read_rslt?;
        ws.write_frame(&mut Frame::new_fin(OpCode::Text, &mut buffer)).await?;
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
        let mut ws = WebSocketServer::accept(
          (),
          Xorshift64::from(simple_seed()),
          stream,
          WebSocketBuffer::default(),
          |_| wtx::Result::Ok(()),
        )
        .await?;
        loop {
          let frame = ws.read_frame().await?;
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
