//! h2load

use wtx::{
  http::{
    HttpRecvParams,
    http2_server_framework::{Http2ServerFramework, HttpRouter, State, get},
  },
  rng::{ChaCha20, CryptoSeedableRng as _},
  tls::TlsConfig,
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let router = HttpRouter::paths(wtx::paths!(("/", get(root)),))?;
  Http2ServerFramework::tokio(ChaCha20::from_std_random()?, TlsConfig::plaintext())?
    .set_http_recv_params(HttpRecvParams::with_permissive_params())
    .run("127.0.0.1:9000", router)
    .await
}

async fn root(_state: State<'_, ()>) -> wtx::Result<()> {
  Ok(())
}
