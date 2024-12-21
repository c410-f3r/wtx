//! WebSocket autobahn client.

use tokio::net::TcpStream;
use wtx::{
  misc::{simple_seed, UriRef, Xorshift64},
  web_socket::{compression::Flate2, Frame, OpCode, WebSocket, WebSocketBuffer},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let host = "127.0.0.1:9080";
  let mut wsb = WebSocketBuffer::default();
  for case in 1..=get_case_count(host, &mut wsb).await? {
    let mut ws = WebSocket::connect(
      Flate2::default(),
      [],
      false,
      Xorshift64::from(simple_seed()),
      TcpStream::connect(host).await?,
      &UriRef::new(&format!("http://{host}/runCase?case={case}&agent=wtx")),
      &mut wsb,
      |_| wtx::Result::Ok(()),
    )
    .await?;
    let (mut common, mut reader, mut writer) = ws.parts_mut();
    loop {
      let mut frame = match reader.read_frame(&mut common).await {
        Err(_err) => {
          ws.write_frame(&mut Frame::new_fin(OpCode::Close, &mut [])).await?;
          break;
        }
        Ok(elem) => elem,
      };
      match frame.op_code() {
        OpCode::Binary | OpCode::Text => writer.write_frame(&mut common, &mut frame).await?,
        OpCode::Close => break,
        _ => {}
      }
    }
  }
  WebSocket::connect(
    (),
    [],
    false,
    Xorshift64::from(simple_seed()),
    TcpStream::connect(host).await?,
    &UriRef::new(&format!("http://{host}/updateReports?agent=wtx")),
    wsb,
    |_| wtx::Result::Ok(()),
  )
  .await?
  .write_frame(&mut Frame::new_fin(OpCode::Close, &mut []))
  .await
}

async fn get_case_count(host: &str, wsb: &mut WebSocketBuffer) -> wtx::Result<u32> {
  let mut ws = WebSocket::connect(
    (),
    [],
    false,
    Xorshift64::from(simple_seed()),
    TcpStream::connect(host).await?,
    &UriRef::new(&format!("http://{host}/getCaseCount")),
    wsb,
    |_| wtx::Result::Ok(()),
  )
  .await?;
  let rslt = ws.read_frame().await?.text_payload().unwrap_or_default().parse()?;
  ws.write_frame(&mut Frame::new_fin(OpCode::Close, &mut [])).await?;
  Ok(rslt)
}
