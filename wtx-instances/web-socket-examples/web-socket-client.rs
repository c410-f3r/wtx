//! WebSocket CLI client that enables real-time communication by allowing users to send and
//! receive messages through typing.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use tokio::net::TcpStream;
use wtx::{
  collection::Vector,
  misc::Uri,
  web_socket::{OpCode, WebSocketConnector},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new("SOME_URI");
  let mut ws = WebSocketConnector::default()
    .headers([("custom-key", "CUSTOM_VALUE")]) // Headers are optional. This method can be omitted.
    .connect(TcpStream::connect(uri.hostname_with_implied_port()).await?, &uri.to_ref())
    .await?;
  let mut buffer = Vector::new();
  loop {
    let frame = ws.read_frame(&mut buffer).await?.0;
    match (frame.op_code(), frame.text_payload()) {
      // `read_frame` internally already sent a Close response
      (OpCode::Close, Some(text)) => {
        println!("Received close frame: {text}");
        break;
      }
      // `read_frame` internally already sent a Pong response
      (OpCode::Ping, _) => {}
      // For any other type, `read_frame` doesn't automatically send frames
      (_, text) => {
        if let Some(elem) = text {
          println!("Received text frame: {elem}")
        }
      }
    }
  }
  Ok(())
}
