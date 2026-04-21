//! Fetches an URI using low-level HTTP/2 resources.
//!
//! # Important
//!
//! #[wtx::main]` shouldn't be used in production environments, as such, replace `#[wtx::main]`
//! with `#[tokio::main]` (tokio), `#[apply(main!)]` (smol) or any other runtime. The associated
//! feature flags (`tokio`, `async-net`, etc.) and streams (`tokio::net::TcpStream`,
//! `async_net::TcpSTream`, etc.) should also be utilized.

extern crate wtx;
extern crate wtx_examples;

use std::net::TcpStream;
use wtx::{
  executor::Runtime,
  http::{HttpClient, ReqBuilder, ReqResBuffer},
  http2::{Http2, Http2Buffer, Http2ErrorCode, Http2Params},
  misc::{Uri, from_utf8_basic},
  rng::{CryptoSeedableRng, Xorshift64},
  sync::Arc,
};

#[wtx::main]
async fn main(runtime: Arc<Runtime>) -> wtx::Result<()> {
  let uri = Uri::new("SOME_URI");
  let stream = TcpStream::connect(uri.hostname_with_implied_port())?;
  let (frame_reader, http2) = Http2::connect(
    Http2Buffer::new(&mut Xorshift64::from_getrandom()?),
    Http2Params::default(),
    (stream.try_clone()?, stream),
  )
  .await?;
  let _jh = runtime.spawn_threaded(frame_reader)?;
  let res = http2.send_req_recv_res(ReqBuilder::get(uri.to_ref()), ReqResBuffer::empty()).await?;
  println!("{}", from_utf8_basic(&res.rrd.body)?);
  http2.send_go_away(Http2ErrorCode::NoError).await;
  Ok(())
}
