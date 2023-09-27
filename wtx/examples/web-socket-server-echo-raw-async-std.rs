//! WebSocket echo server.

#[path = "./common/mod.rs"]
mod common;

use async_std::net::TcpListener;
use wtx::{web_socket::FrameBufferVec, PartitionedBuffer};

fn main() -> wtx::Result<()> {
  async_std::task::block_on::<_, wtx::Result<_>>(async {
    let listener = TcpListener::bind(common::_host_from_args()).await?;
    loop {
      let (stream, _) = listener.accept().await?;
      let _jh = async_std::task::spawn(async move {
        if let Err(err) = common::_accept_conn_and_echo_frames(
          (),
          &mut FrameBufferVec::default(),
          &mut PartitionedBuffer::default(),
          stream,
        )
        .await
        {
          println!("{err}");
        }
      });
    }
  })?;
  Ok(())
}
