//! WebSocket autobahn client.

use tokio::net::TcpStream;
use wtx::{
  misc::{simple_seed, UriRef, Xorshift64},
  web_socket::{
    compression::Flate2, CloseCode, FrameBufferVec, FrameMutVec, HeadersBuffer, OpCode,
    WebSocketBuffer, WebSocketClient,
  },
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let fb = &mut FrameBufferVec::default();
  let host = "127.0.0.1:9080";
  let mut wsb = WebSocketBuffer::default();
  for case in 1..=get_case_count(fb, host, &mut wsb).await? {
    let (_, mut ws) = WebSocketClient::connect(
      Flate2::default(),
      fb,
      [],
      &mut HeadersBuffer::default(),
      Xorshift64::from(simple_seed()),
      TcpStream::connect(host).await?,
      &UriRef::new(&format!("http://{host}/runCase?case={case}&agent=wtx")),
      &mut wsb,
    )
    .await?;
    loop {
      let mut frame = match ws.read_frame(fb).await {
        Err(error) => {
          eprintln!("Error: {error}");
          ws.write_frame(&mut FrameMutVec::new_fin(fb, OpCode::Close, &[])?).await?;
          break;
        }
        Ok(elem) => elem,
      };
      match frame.op_code() {
        OpCode::Binary | OpCode::Text => ws.write_frame(&mut frame).await?,
        OpCode::Close => break,
        _ => {}
      }
    }
  }
  WebSocketClient::connect(
    (),
    fb,
    [],
    &mut HeadersBuffer::default(),
    Xorshift64::from(simple_seed()),
    TcpStream::connect(host).await?,
    &UriRef::new(&format!("http://{host}/updateReports?agent=wtx")),
    wsb,
  )
  .await?
  .1
  .write_frame(&mut FrameMutVec::close_from_params(CloseCode::Normal, fb, &[])?)
  .await
}

async fn get_case_count(
  fb: &mut FrameBufferVec,
  host: &str,
  wsb: &mut WebSocketBuffer,
) -> wtx::Result<u32> {
  let (_, mut ws) = WebSocketClient::connect(
    (),
    fb,
    [],
    &mut HeadersBuffer::default(),
    Xorshift64::from(simple_seed()),
    TcpStream::connect(host).await?,
    &UriRef::new(&format!("http://{host}/getCaseCount")),
    wsb,
  )
  .await?;
  let rslt = ws.read_frame(fb).await?.text_payload().unwrap_or_default().parse()?;
  ws.write_frame(&mut FrameMutVec::close_from_params(CloseCode::Normal, fb, &[])?).await?;
  Ok(rslt)
}
