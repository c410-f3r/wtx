//! Fetches and prints the response body of a provided URI.

extern crate tokio;
extern crate wtx;

use wtx::{
  collections::Vector,
  executor::TokioExecutor,
  http::{HttpClient, ReqBuilder, http2_client_pool::Http2ClientPoolBuilder},
  misc::{Uri, from_utf8_basic},
  rng::{ChaCha20, CryptoSeedableRng as _},
  tls::{TlsConfig, TlsModeVerified},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new("https://github.com/c410-f3r/wtx");
  let res = Http2ClientPoolBuilder::new(
    TokioExecutor::default(),
    1,
    ChaCha20::from_getrandom()?,
    TlsConfig::from_ccadb(TlsModeVerified::default())?.into(),
  )?
  .build()
  .send_req_recv_res(&mut Vector::new(), ReqBuilder::get(uri.to_ref()).into_request())
  .await?;
  println!("{}", from_utf8_basic(&res.msg_data.body)?);
  Ok(())
}
