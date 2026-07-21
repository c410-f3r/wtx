//! Fetches an URI using low-level HTTP/2 resources.

extern crate tokio;
extern crate wtx;

use wtx::{
  collections::Vector,
  http::{HttpClient, HttpRecvParams, ReqBuilder},
  http2::{Http2, Http2Buffer, Http2ErrorCode},
  misc::from_utf8_basic,
  net::{Stream, Uri},
  rng::{ChaCha20, CryptoSeedableRng},
  tls::{TlsConfig, TlsConnectorBuilder, TlsModeVerified},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new("https://github.com/c410-f3r/wtx");
  let mut rng = ChaCha20::from_getrandom()?;
  let hb = Http2Buffer::new(&mut rng);
  let hrp = HttpRecvParams::with_optioned_params();
  let tls_config = TlsConfig::from_ccadb(TlsModeVerified::default())?;
  let tcr = TlsConnectorBuilder::tokio(&uri).build(tls_config, rng).await?.connect().await?;
  let (frame_reader, http2) = Http2::connect(hb, hrp, tcr.tls_stream.into_split()?).await?;
  let _jh = tokio::spawn(frame_reader);
  let res = http2
    .send_req_recv_res(&mut Vector::new(), ReqBuilder::get(uri.to_ref()).into_request())
    .await?;
  println!("{}", from_utf8_basic(&res.msg_data.body)?);
  http2.send_go_away(Http2ErrorCode::NoError).await;
  Ok(())
}
