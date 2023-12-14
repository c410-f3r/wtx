//! WebSocket autobahn server.

#[path = "../../examples/common/mod.rs"]
mod common;

use tokio::net::TcpListener;
use wtx::web_socket::compression::Flate2;

#[tokio::main]
async fn main() {
  let listener = TcpListener::bind("127.0.0.1:9070").await.unwrap();
  loop {
    let (stream, _) = listener.accept().await.unwrap();
    let _jh = tokio::spawn(async move {
      let _rslt =
        common::_accept_conn_and_echo_frames(Flate2::default(), &mut <_>::default(), stream).await;
    });
  }
}
