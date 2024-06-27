//! Http2 CLI client

#[path = "./common/mod.rs"]
mod common;

use std::net::ToSocketAddrs;
use wtx::{
  http::{Method, RequestStr},
  http2::{Http2Buffer, Http2ErrorCode, Http2Params, Http2Tokio, StreamBuffer},
  misc::{from_utf8_basic, TokioRustlsConnector, UriString},
  rng::StaticRng,
};

#[tokio::main]
async fn main() {
  let uri = UriString::new(common::_uri_from_args());
  let mut sb = StreamBuffer::default();
  let mut http2 = Http2Tokio::connect(
    Http2Buffer::new(StaticRng::default()),
    Http2Params::default(),
    TokioRustlsConnector::from_webpki_roots()
      .http2()
      .with_tcp_stream(uri.host().to_socket_addrs().unwrap().next().unwrap(), uri.hostname())
      .await
      .unwrap(),
  )
  .await
  .unwrap();
  let mut stream = http2.stream().await.unwrap();
  stream
    .send_req(&mut sb.hpack_enc_buffer, RequestStr::http2(b"", Method::Get, uri.to_ref()))
    .await
    .unwrap();
  let (res_sb, _status_code) = stream.recv_res(sb).await.unwrap();
  println!("{}", from_utf8_basic(&res_sb.rrb.body).unwrap());
  http2.send_go_away(Http2ErrorCode::NoError).await;
}
