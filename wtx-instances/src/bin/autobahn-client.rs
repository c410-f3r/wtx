//! WebSocket autobahn client.

use tokio::net::TcpStream;
use wtx::{
  collection::Vector,
  web_socket::{Frame, OpCode, WebSocketConnector, compression::Flate2},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let host = "127.0.0.1:9080";
  let mut buffer = Vector::new();
  for case in 1..=get_case_count(&mut buffer, host).await? {
    let mut ws = WebSocketConnector::default()
      .compression(Flate2::default())
      .no_masking(false)
      .connect(
        TcpStream::connect(host).await?,
        &format!("http://{host}/runCase?case={case}&agent=wtx").as_str().into(),
      )
      .await?;
    let (mut common, mut reader, mut writer) = ws.parts_mut();
    loop {
      let mut frame = match reader.read_frame(&mut buffer, &mut common).await {
        Err(_err) => {
          ws.write_frame(&mut Frame::new_fin(OpCode::Close, &mut [])).await?;
          break;
        }
        Ok(elem) => elem.0,
      };
      match frame.op_code() {
        OpCode::Binary | OpCode::Text => writer.write_frame(&mut common, &mut frame).await?,
        OpCode::Close => break,
        _ => {}
      }
    }
  }
  WebSocketConnector::default()
    .connect(
      TcpStream::connect(host).await?,
      &format!("http://{host}/updateReports?agent=wtx").as_str().into(),
    )
    .await?
    .write_frame(&mut Frame::new_fin(OpCode::Close, &mut []))
    .await
}

async fn get_case_count(buffer: &mut Vector<u8>, host: &str) -> wtx::Result<u32> {
  let mut ws = WebSocketConnector::default()
    .connect(
      TcpStream::connect(host).await?,
      &format!("http://{host}/getCaseCount").as_str().into(),
    )
    .await?;
  let rslt = ws.read_frame(buffer).await?.0.text_payload().unwrap_or_default().parse()?;
  ws.write_frame(&mut Frame::new_fin(OpCode::Close, &mut [])).await?;
  Ok(rslt)
}
