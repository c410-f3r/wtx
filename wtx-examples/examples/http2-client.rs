//! Fetches an URI using low-level HTTP/2 resources.

extern crate tokio;
extern crate wtx;

use tokio::net::TcpStream;
use wtx::{
  collections::Vector,
  http::{HttpClient, HttpRecvParams, ReqBuilder},
  http2::{Http2, Http2Buffer, Http2ErrorCode},
  misc::{Uri, from_utf8_basic},
  rng::{ChaCha20, CryptoSeedableRng},
  stream::Stream,
  tls::{TlsConfig, TlsConnector, TlsModeVerified},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new("https://github.com/c410-f3r/wtx");
  let stream = TcpStream::connect(uri.hostname_with_implied_port()).await?;
  let mut rng = ChaCha20::from_getrandom()?;
  let hb = Http2Buffer::new(&mut rng);
  let hrp = HttpRecvParams::with_optioned_params();
  let tls_config = TlsConfig::from_ccadb(TlsModeVerified::default())?;
  let tcr = TlsConnector::new(tls_config, rng, stream).connect().await?.rslt()?;
  let (frame_reader, http2) = Http2::connect(hb, hrp, tcr.tls_stream.into_split()?).await?;
  let _jh = tokio::spawn(frame_reader);
  let res = http2
    .send_req_recv_res(&mut Vector::new(), ReqBuilder::get(uri.to_ref()).into_request())
    .await?;
  println!("{}", from_utf8_basic(&res.msg_data.body)?);
  http2.send_go_away(Http2ErrorCode::NoError).await;
  Ok(())
}
