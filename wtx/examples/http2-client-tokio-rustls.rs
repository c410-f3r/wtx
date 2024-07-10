//! Http2 CLI client

#[path = "./common/mod.rs"]
mod common;

use wtx::{
  http::{Method, ReqResBuffer, ReqUri, Request},
  http2::{Http2Buffer, Http2ErrorCode, Http2Params, Http2Tokio},
  misc::{from_utf8_basic, TokioRustlsConnector, UriString},
  rng::StaticRng,
};

#[tokio::main]
async fn main() {
  let uri = UriString::new(common::_uri_from_args());
  let mut http2 = Http2Tokio::connect(
    Http2Buffer::new(StaticRng::default()),
    Http2Params::default(),
    TokioRustlsConnector::from_webpki_roots()
      .with_tcp_stream(uri.host(), uri.hostname())
      .await
      .unwrap(),
  )
  .await
  .unwrap();
  let mut stream = http2.stream().await.unwrap();
  stream.send_req(Request::http2(Method::Get, &uri), ReqUri::Data).await.unwrap();
  let rrb = ReqResBuffer::default();
  let (res_rrb, _status_code) = stream.recv_res(rrb).await.unwrap();
  println!("{}", from_utf8_basic(res_rrb.body()).unwrap());
  http2.send_go_away(Http2ErrorCode::NoError).await;
}
