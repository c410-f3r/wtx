//! WebSocket echo server.

#[path = "./common/mod.rs"]
mod common;

use smol::net::TcpListener;

fn main() {
  smol::block_on(async {
    let listener = TcpListener::bind(common::_host_from_args()).await.unwrap();
    loop {
      let (stream, _) = listener.accept().await.unwrap();
      smol::spawn(async move {
        common::_accept_conn_and_echo_frames((), &mut <_>::default(), stream).await.unwrap();
      })
      .detach();
    }
  });
}
