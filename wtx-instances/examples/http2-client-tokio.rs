//! Fetches an URI using low-level HTTP/2 resources.
//!
//! This snippet requires ~25 dependencies and has an optimized binary size of ~700K.
//!
//! USAGE: `./program http://www.example.com:80`

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use tokio::net::TcpStream;
use wtx::{
  http::{Method, ReqResBuffer, Request},
  http2::{Http2Buffer, Http2ErrorCode, Http2Params, Http2Tokio},
  misc::{from_utf8_basic, Either, NoStdRng, Uri},
};

#[tokio::main]
async fn main() {
  let uri = Uri::new(wtx_instances::uri_from_args());
  let (frame_reader, mut http2) = Http2Tokio::connect(
    Http2Buffer::new(NoStdRng::default()),
    Http2Params::default(),
    TcpStream::connect(uri.host()).await.unwrap().into_split(),
  )
  .await
  .unwrap();
  let _jh = tokio::spawn(async move {
    frame_reader.await.unwrap();
  });
  let rrb = ReqResBuffer::default();
  let mut stream = http2.stream().await.unwrap();
  stream.send_req(Request::http2(Method::Get, b"Hello!"), &uri.to_ref()).await.unwrap().unwrap();
  let Either::Right(res) = stream.recv_res(rrb).await.unwrap() else {
    panic!();
  };
  println!("{}", from_utf8_basic(&res.0.data).unwrap());
  http2.send_go_away(Http2ErrorCode::NoError).await;
}
