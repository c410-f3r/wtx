//! WebSocket autobahn server.

#[path = "../../examples/common/mod.rs"]
mod common;

use tokio::net::TcpListener;
use wtx::web_socket::compression::Flate2;

#[tokio::main(flavor = "current_thread")]
async fn main() -> wtx::Result<()> {
  let listener = TcpListener::bind("127.0.0.1:9070").await?;
  loop {
    let (stream, _) = listener.accept().await?;
    let _jh = tokio::spawn(async move {
      if let Err(err) = tokio::task::unconstrained(common::_accept_conn_and_echo_frames(
        Flate2::default(),
        &mut <_>::default(),
        &mut <_>::default(),
        stream,
      ))
      .await
      {
        println!("{err}");
      }
    });
  }
}
