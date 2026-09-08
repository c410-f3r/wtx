//! testssl

use wtx::{
  http::http2_server_framework::{Http2ServerFramework, HttpRouter, VerbatimParams, get},
  tls::TlsConfig,
};

pub static FULL_CHAIN: &[u8] = include_bytes!("../../../.certs/fullchain.pem");
pub static SECRET_KEY: &[u8] = include_bytes!("../../../.certs/key.pem");

fn main() -> wtx::Result<()> {
  let tls_config = TlsConfig::from_keys_pem(FULL_CHAIN, SECRET_KEY)?;
  let router = HttpRouter::paths(wtx::paths!(("/", get(root)),))?;
  Http2ServerFramework::tokio(tls_config)?.run_in_threads("0.0.0.0:9000", router)
}

async fn root() -> wtx::Result<VerbatimParams> {
  Ok(VerbatimParams::default())
}
