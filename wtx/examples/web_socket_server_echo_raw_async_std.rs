//! WebSocket echo server.

mod common;

use async_std::net::TcpListener;
use wtx::{web_socket::FrameBufferVec, ReadBuffer};

#[async_std::main]
async fn main() -> wtx::Result<()> {
    let listener = TcpListener::bind(common::_host_from_args()).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let _jh = async_std::task::spawn(async move {
            if let Err(err) = common::_accept_conn_and_echo_frames(
                &mut FrameBufferVec::default(),
                &mut ReadBuffer::default(),
                stream,
            )
            .await
            {
                println!("{err}");
            }
        });
    }
}
