//! WebSocket CLI client that enables real-time communication by allowing users to send and
//! receive messages through typing.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use tokio::{
  io::{AsyncBufReadExt, BufReader},
  net::TcpStream,
};
use wtx::{
  misc::Uri,
  web_socket::{Frame, OpCode, WebSocketConnector},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new("SOME_URI");
  let mut ws = WebSocketConnector::default()
    .headers([("custom-key", "CUSTOM_VALUE")]) // Headers are optional. This method can be omitted.
    .connect(TcpStream::connect(uri.hostname_with_implied_port()).await?, &uri.to_ref())
    .await?;
  let mut buffer = Vec::new();
  let mut reader = BufReader::new(tokio::io::stdin());
  loop {
    tokio::select! {
      frame_rslt = ws.read_frame() => {
        let frame = frame_rslt?;
        match (frame.op_code(), frame.text_payload()) {
          (_, Some(elem)) => println!("{elem}"),
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
