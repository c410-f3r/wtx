//! Fetches an URI using low-level HTTP/2 resources.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use tokio::net::TcpStream;
use wtx::{
  http::{HttpClient, Method, ReqResBuffer},
  http2::{Http2Buffer, Http2ErrorCode, Http2Params, Http2Tokio},
  misc::{Uri, from_utf8_basic},
  rng::{Xorshift64, simple_seed},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new("SOME_URI");
  let (frame_reader, http2) = Http2Tokio::connect(
    Http2Buffer::new(&mut Xorshift64::from(simple_seed())),
    Http2Params::default(),
    TcpStream::connect(uri.hostname_with_implied_port()).await?.into_split(),
  )
  .await?;
  let _jh = tokio::spawn(frame_reader);
  let res = http2.send_recv_single(Method::Get, ReqResBuffer::empty(), &uri).await?;
  println!("{}", from_utf8_basic(&res.rrd.body)?);
  http2.send_go_away(Http2ErrorCode::NoError).await;
  Ok(())
}
