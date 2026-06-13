//! Fetches an URI using low-level HTTP/2 resources.

extern crate tokio;
extern crate wtx;
extern crate wtx_examples;

use tokio::net::TcpStream;
use wtx::{
  http::{HttpClient, HttpRecvParams, ReqBuilder},
  http2::{Http2, Http2Buffer, Http2ErrorCode},
  misc::{Uri, from_utf8_basic},
  rng::{CryptoSeedableRng, Xorshift64},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new("SOME_URI");
  let stream = TcpStream::connect(uri.hostname_with_implied_port()).await?;
  let (frame_reader, http2) = Http2::connect(
    Http2Buffer::new(&mut Xorshift64::from_getrandom()?),
    HttpRecvParams::with_optioned_params(),
    stream.into_split(),
  )
  .await?;
  let _jh = tokio::spawn(frame_reader);
  let res = http2.send_req_recv_res(ReqBuilder::get(uri.to_ref()).into_request()).await?;
  println!("{}", from_utf8_basic(&res.msg_data.body)?);
  http2.send_go_away(Http2ErrorCode::NoError).await;
  Ok(())
}
