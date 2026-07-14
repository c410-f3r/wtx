//! testssl

use wtx::{
  http::http2_server_framework::{Http2ServerFramework, HttpRouter, get},
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
  .run_in_threads("127.0.0.1:9000", HttpRouter::paths(wtx::paths!(("/", get(root)),))?)
}

async fn root() -> &'static str {
  "Hello"
}
