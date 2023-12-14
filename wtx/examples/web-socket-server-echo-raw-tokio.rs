//! WebSocket echo server.

#[path = "./common/mod.rs"]
mod common;

use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
  let listener = TcpListener::bind(common::_host_from_args()).await.unwrap();
  loop {
    let (stream, _) = listener.accept().await.unwrap();
    let _jh = tokio::spawn(async move {
      common::_accept_conn_and_echo_frames((), &mut <_>::default(), stream).await
    });
  }
}
