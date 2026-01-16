//! Fetches an URI using low-level HTTP/2 resources.

extern crate tokio;
extern crate wtx;
extern crate wtx_instances;

use tokio::net::TcpStream;
use wtx::{
  http::{HttpClient, ReqBuilder, ReqResBuffer},
  http2::{Http2, Http2Buffer, Http2ErrorCode, Http2Params},
  misc::{Uri, from_utf8_basic},
  rng::{ChaCha20, SeedableRng as _, Xorshift64, simple_seed},
  tls::{TlsConfig, TlsConnector},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new("SOME_URI");
  let mut rng = ChaCha20::from_std_random()?;
  let tls_stream = TlsConnector::default()
    .connect(
      &mut rng,
      TcpStream::connect(uri.hostname_with_implied_port()).await?,
      &TlsConfig::default(),
    )
    .await?;
  let (frame_reader, http2) = Http2::connect(
    Http2Buffer::new(&mut Xorshift64::from(simple_seed())),
    Http2Params::default(),
    tls_stream.into_split(TcpStream::into_split),
  )
  .await?;
  let _jh = tokio::spawn(frame_reader);
  let res = http2.send_req_recv_res(ReqResBuffer::empty(), ReqBuilder::get(uri.to_ref())).await?;
  println!("{}", from_utf8_basic(&res.rrd.body)?);
  http2.send_go_away(Http2ErrorCode::NoError).await;
  Ok(())
}
