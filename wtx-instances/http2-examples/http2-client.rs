//! Fetches an URI using low-level HTTP/2 resources.
//!
//! This snippet requires ~25 dependencies and has an optimized binary size of ~700K.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use tokio::net::TcpStream;
use wtx::{
  http::{Method, ReqResBuffer, Request},
  http2::{Http2Buffer, Http2ErrorCode, Http2Params, Http2Tokio},
  misc::{Uri, Xorshift64, from_utf8_basic, simple_seed},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new("SOME_URI");
  let (frame_reader, mut http2) = Http2Tokio::connect(
    Http2Buffer::new(Xorshift64::from(simple_seed())),
    Http2Params::default(),
    TcpStream::connect(uri.hostname_with_implied_port()).await?.into_split(),
  )
  .await?;
  let _jh = tokio::spawn(frame_reader);
  let rrb = ReqResBuffer::empty();
  let mut stream = http2.stream().await?;
  stream.send_req(Request::http2(Method::Get, b"Hello!"), &uri.to_ref()).await?;
  let (_, res_rrb) = stream.recv_res(rrb).await?;
  stream.common().clear(false).await?;
  println!("{}", from_utf8_basic(&res_rrb.body)?);
  http2.send_go_away(Http2ErrorCode::NoError).await;
  Ok(())
}
