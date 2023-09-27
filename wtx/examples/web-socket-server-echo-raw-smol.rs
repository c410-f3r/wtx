//! WebSocket echo server.

#[path = "./common/mod.rs"]
mod common;

use smol::net::TcpListener;

fn main() -> wtx::Result<()> {
  smol::block_on::<wtx::Result<_>>(async {
    let listener = TcpListener::bind(common::_host_from_args()).await?;
    loop {
      let (stream, _) = listener.accept().await?;
      smol::spawn(async move {
        if let Err(err) =
          common::_accept_conn_and_echo_frames((), &mut <_>::default(), &mut <_>::default(), stream)
            .await
        {
          println!("{err}");
        }
      })
      .detach();
    }
  })?;
  Ok(())
}
