//! WebSocket autobahn client.

use tokio::net::TcpStream;
use wtx::{
  misc::UriRef,
  rng::StdRng,
  web_socket::{
    compression::Flate2,
    handshake::{HeadersBuffer, WebSocketConnect, WebSocketConnectRaw},
    CloseCode, FrameBufferVec, FrameMutVec, OpCode, WebSocketBuffer,
  },
};

#[tokio::main]
async fn main() {
  let fb = &mut FrameBufferVec::default();
  let host = "127.0.0.1:9080";
  let mut wsb = WebSocketBuffer::default();
  for case in 1..=get_case_count(fb, host, &mut wsb).await {
    let (_, mut ws) = WebSocketConnectRaw {
      compression: Flate2::default(),
      fb,
      headers_buffer: &mut HeadersBuffer::default(),
      rng: StdRng::default(),
      stream: TcpStream::connect(host).await.unwrap(),
      uri: &UriRef::new(&format!("http://{host}/runCase?case={case}&agent=wtx")),
      wsb: &mut wsb,
    }
    .connect([])
    .await
    .unwrap();
    loop {
      let mut frame = match ws.read_frame(fb).await {
        Err(err) => {
          println!("Error: {err}");
          ws.write_frame(&mut FrameMutVec::new_fin(fb, OpCode::Close, &[]).unwrap()).await.unwrap();
          break;
        }
        Ok(elem) => elem,
      };
      match frame.op_code() {
        OpCode::Binary | OpCode::Text => ws.write_frame(&mut frame).await.unwrap(),
        OpCode::Close => break,
        _ => {}
      }
    }
  }
  WebSocketConnectRaw {
    compression: (),
    fb,
    headers_buffer: &mut HeadersBuffer::default(),
    rng: StdRng::default(),
    stream: TcpStream::connect(host).await.unwrap(),
    uri: &UriRef::new(&format!("http://{host}/updateReports?agent=wtx")),
    wsb,
  }
  .connect([])
  .await
  .unwrap()
  .1
  .write_frame(&mut FrameMutVec::close_from_params(CloseCode::Normal, fb, &[]).unwrap())
  .await
  .unwrap();
}

async fn get_case_count(fb: &mut FrameBufferVec, host: &str, wsb: &mut WebSocketBuffer) -> u32 {
  let (_, mut ws) = WebSocketConnectRaw {
    compression: (),
    fb,
    headers_buffer: &mut HeadersBuffer::default(),
    rng: StdRng::default(),
    stream: TcpStream::connect(host).await.unwrap(),
    uri: &UriRef::new(&format!("http://{host}/getCaseCount")),
    wsb,
  }
  .connect([])
  .await
  .unwrap();
  let rslt = ws.read_frame(fb).await.unwrap().text_payload().unwrap_or_default().parse().unwrap();
  ws.write_frame(&mut FrameMutVec::close_from_params(CloseCode::Normal, fb, &[]).unwrap())
    .await
    .unwrap();
  rslt
}
