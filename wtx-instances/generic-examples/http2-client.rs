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
  misc::{from_utf8_basic, Either, NoStdRng, Uri},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new("http://www.example.com:80");
  let (frame_reader, mut http2) = Http2Tokio::connect(
    Http2Buffer::new(NoStdRng::default()),
    Http2Params::default(),
    TcpStream::connect(uri.host()).await?.into_split(),
  )
  .await?;
  let _jh = tokio::spawn(async move {
    if let Err(err) = frame_reader.await {
      eprintln!("{err}");
    }
  });
  let rrb = ReqResBuffer::default();
  let mut stream = http2.stream().await?;
  stream.send_req(Request::http2(Method::Get, b"Hello!"), &uri.to_ref()).await?;
  let Either::Right(res) = stream.recv_res(rrb).await? else {
    return Err(wtx::Error::ClosedConnection);
  };
  println!("{}", from_utf8_basic(&res.0.data)?);
  http2.send_go_away(Http2ErrorCode::NoError).await;
  Ok(())
}
