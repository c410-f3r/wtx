//! WebSocket echo server.

#[path = "./common/mod.rs"]
mod common;

use monoio::net::TcpListener;

#[monoio::main]
async fn main() -> wtx::Result<()> {
  let listener = TcpListener::bind(common::_host_from_args())?;
  loop {
    let (stream, _) = listener.accept().await?;
    let _jh = monoio::spawn(async move {
      if let Err(err) =
        common::_accept_conn_and_echo_frames((), &mut <_>::default(), &mut <_>::default(), stream)
          .await
      {
        println!("{err}");
      }
    });
  }
}
