//! Low-level HTTP/2 server that servers a single response.

extern crate tokio;
extern crate wtx;
extern crate wtx_examples;

use tokio::net::TcpListener;
use wtx::{
  http::{HttpRecvParams, Response, StatusCode},
  http2::{Http2, Http2Buffer, Http2ErrorCode, Http2RecvStatus},
  misc::Uri,
  rng::{CryptoSeedableRng, Xorshift64},
};
use wtx_examples::host_from_args;

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new(host_from_args());
  let listener = TcpListener::bind(uri.hostname_with_implied_port()).await?;
  let (stream, _) = listener.accept().await?;
  let (frame_reader, http2) = Http2::accept(
    Http2Buffer::new(&mut Xorshift64::from_getrandom()?),
    HttpRecvParams::with_optioned_params(),
    stream.into_split(),
  )
  .await?;
  let _jh = tokio::spawn(frame_reader);
  let Some((mut stream, _)) = http2.stream(|_, _| {}).await? else {
    println!("Connection closed!");
    return Ok(());
  };
  let (hrs, msg) = stream.recv_req().await?;
  if let Http2RecvStatus::ClosedConnection | Http2RecvStatus::ClosedStream(_) = hrs {
    println!("Connection or stream closed!");
    return Ok(());
  }
  println!("An arbitrary request has been received: {msg:#?}");
  let _ = stream.send_res(Response::new(b"By tea, for tea\n", StatusCode::ImATeapot)).await?;
  http2.send_go_away(Http2ErrorCode::NoError).await;
  Ok(())
}
