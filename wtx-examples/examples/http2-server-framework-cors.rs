//! The `CorsMiddleware` middleware inserts permissive CORS headers in every response.

use wtx::{
  http::http2_server_framework::{CorsMiddleware, Http2ServerFramework, HttpRouter, get},
  rng::{ChaCha20, CryptoSeedableRng},
  tls::{TlsConfig, TlsModeVerified},
};
use wtx_examples::{PUBLIC_KEY, SECRET_KEY, host_from_args};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  Http2ServerFramework::tokio(
    ChaCha20::from_getrandom()?,
    TlsConfig::from_keys_pem(
      TlsModeVerified::default(),
      PUBLIC_KEY.try_into()?,
      SECRET_KEY.try_into()?,
    )?,
  )?
  .run(
    &host_from_args(),
    HttpRouter::new(wtx::paths!(("/hello", get(hello))), CorsMiddleware::permissive())?,
  )
  .await
}

async fn hello() -> &'static str {
  "Hello"
}
