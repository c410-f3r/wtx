//! WebSocket echo server.

mod common;

use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> wtx::Result<()> {
    let listener = TcpListener::bind(common::_host_from_args()).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let _jh = tokio::spawn(async move {
            if let Err(err) = tokio::task::unconstrained(common::_accept_conn_and_echo_frames(
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
