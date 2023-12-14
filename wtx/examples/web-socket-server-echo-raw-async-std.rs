//! WebSocket echo server.

#[path = "./common/mod.rs"]
mod common;

use async_std::net::TcpListener;
use wtx::web_socket::FrameBufferVec;

fn main() {
  async_std::task::block_on(async {
    let listener = TcpListener::bind(common::_host_from_args()).await.unwrap();
    loop {
      let (stream, _) = listener.accept().await.unwrap();
      let _jh = async_std::task::spawn(async move {
        common::_accept_conn_and_echo_frames((), &mut FrameBufferVec::default(), stream).await
      });
    }
  });
}
