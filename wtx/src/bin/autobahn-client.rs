//! WebSocket autobahn client.

use tokio::net::TcpStream;
use wtx::{
  rng::StdRng,
  web_socket::{
    compression::Flate2, handshake::WebSocketConnectRaw, CloseCode, FrameBufferVec, FrameMutVec,
    OpCode, WebSocketClient,
  },
  PartitionedBuffer,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
  let fb = &mut <_>::default();
  let host = "127.0.0.1:9080";
  let pb = &mut PartitionedBuffer::default();
  for case in 1..=get_case_count(fb, &host, pb).await? {
    let (_, mut ws) = WebSocketClient::connect(WebSocketConnectRaw {
      compression: Flate2::default(),
      fb,
      headers_buffer: &mut <_>::default(),
      pb: &mut *pb,
      rng: StdRng::default(),
      stream: TcpStream::connect(host).await.map_err(wtx::Error::from)?,
      uri: &format!("http://{host}/runCase?case={case}&agent=wtx"),
    })
    .await?;
    loop {
      let mut frame = match ws.read_frame(fb).await {
        Err(err) => {
          println!("Error: {err}");
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
  WebSocketClient::connect(WebSocketConnectRaw {
    compression: (),
    fb,
    headers_buffer: &mut <_>::default(),
    pb,
    rng: StdRng::default(),
    stream: TcpStream::connect(host).await.map_err(wtx::Error::from)?,
    uri: &format!("http://{host}/updateReports?agent=wtx"),
  })
  .await?
  .1
  .write_frame(&mut FrameMutVec::close_from_params(CloseCode::Normal, fb, &[])?)
  .await?;
  Ok(())
}

/// Error
#[derive(Debug)]
pub enum Error {
  /// ParseIntError
  ParseIntError(std::num::ParseIntError),
  /// Wtx
  Wtx(wtx::Error),
}

impl From<std::num::ParseIntError> for Error {
  fn from(from: std::num::ParseIntError) -> Self {
    Self::ParseIntError(from)
  }
}

impl From<wtx::Error> for Error {
  fn from(from: wtx::Error) -> Self {
    Self::Wtx(from)
  }
}

async fn get_case_count(
  fb: &mut FrameBufferVec,
  host: &str,
  pb: &mut PartitionedBuffer,
) -> Result<u32, Error> {
  let (_, mut ws) = WebSocketClient::connect(WebSocketConnectRaw {
    compression: (),
    fb,
    headers_buffer: &mut <_>::default(),
    pb,
    rng: StdRng::default(),
    stream: TcpStream::connect(host).await.map_err(wtx::Error::from)?,
    uri: &&format!("http://{host}/getCaseCount"),
  })
  .await?;
  let rslt = ws.read_frame(fb).await?.text_payload().unwrap_or_default().parse()?;
  ws.write_frame(&mut FrameMutVec::close_from_params(CloseCode::Normal, fb, &[])?).await?;
  Ok(rslt)
}
