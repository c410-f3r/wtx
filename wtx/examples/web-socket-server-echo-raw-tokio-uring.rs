//! WebSocket echo server.

#[path = "./common/mod.rs"]
mod common;

use tokio_uring::net::TcpListener;

fn main() -> wtx::Result<()> {
  tokio_uring::start(async {
    let listener = TcpListener::bind(common::_host_from_args().as_str().parse().unwrap())?;
    loop {
      let (stream, _) = listener.accept().await?;
      let _jh = tokio_uring::spawn(async move {
        if let Err(err) =
          common::_accept_conn_and_echo_frames((), &mut <_>::default(), &mut <_>::default(), stream)
            .await
        {
          println!("{err}");
        }
      });
    }
  })
}
