//! Http2 server.

#[path = "./common/mod.rs"]
mod common;

use tokio::net::TcpStream;
use wtx::{
  http::{Headers, Method, Request},
  http2::{ConnectParams, Http2Buffer, Http2Tokio, ReqResBuffer},
  misc::{from_utf8_basic, Uri, UriString},
  rng::StaticRng,
};

#[tokio::main]
async fn main() {
  let cp = ConnectParams::default();
  let uri = UriString::new(common::_uri_from_args());
  let mut http2 = Http2Tokio::connect(
    cp,
    Http2Buffer::with_capacity(StaticRng::default()),
    TcpStream::connect(uri.host()).await.unwrap(),
  )
  .await
  .unwrap();
  let mut rrb = ReqResBuffer::default();
  let stream = http2.stream().await.unwrap();
  let res = stream
    .send_req(
      Request::http2(&(), &Headers::new(0), Method::Get, Uri::new("https://http2.akamai.com")),
      &mut rrb,
    )
    .await
    .unwrap();
  println!("{}", from_utf8_basic(res.body()).unwrap())
}
