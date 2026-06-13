//! Fetches and prints the response body of a provided URI.
//!
//! Currently, only HTTP/2 is supported.

extern crate tokio;
extern crate wtx;
extern crate wtx_examples;

use wtx::{
  executor::TokioExecutor,
  http::{HttpClient, ReqBuilder, http2_client_pool::Http2ClientPoolBuilder},
  misc::{Uri, from_utf8_basic},
  rng::{ChaCha20, CryptoSeedableRng},
  tls::TlsConfig,
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let uri = Uri::new("https://github.com/c410-f3r/wtx");
  let res = Http2ClientPoolBuilder::new(TokioExecutor, 1, TlsConfig::from_ccadb())
    .build(ChaCha20::from_std_random()?)
    .send_req_recv_res(ReqBuilder::get(uri.to_ref()).into_request())
    .await?;
  println!("{}", from_utf8_basic(&res.msg_data.body)?);
  Ok(())
}
