//! testssl

use wtx::{
  http::http2_server_framework::{Http2ServerFramework, HttpRouter, State, VerbatimParams, get},
  tls::{TlsConfig, TlsModeVerified},
};

pub static FULL_CHAIN: &[u8] = include_bytes!("../../../.certs/fullchain.pem");
pub static SECRET_KEY: &[u8] = include_bytes!("../../../.certs/key.pem");

fn main() -> wtx::Result<()> {
  Http2ServerFramework::tokio(TlsConfig::from_keys_pem(
    TlsModeVerified::default(),
    FULL_CHAIN.try_into()?,
    SECRET_KEY.try_into()?,
  )?)?
  .run_in_threads("0.0.0.0:9000", HttpRouter::paths(wtx::paths!(("/", get(root)),))?)
}

async fn root(_: State<'_, ()>) -> wtx::Result<VerbatimParams> {
  Ok(VerbatimParams::default())
}
