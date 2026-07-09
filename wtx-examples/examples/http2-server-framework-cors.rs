//! The `CorsMiddleware` middleware inserts permissive CORS headers in every response.

use wtx::{
  http::http2_server_framework::{CorsMiddleware, Http2ServerFramework, HttpRouter, get},
  tls::{TlsConfig, TlsModeVerified},
};
use wtx_examples::{PUBLIC_KEY, SECRET_KEY, host_from_args};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  Http2ServerFramework::tokio(TlsConfig::from_keys_pem(
    TlsModeVerified::default(),
    PUBLIC_KEY.try_into()?,
    SECRET_KEY.try_into()?,
  )?)?
  .set_error_cb(|err| eprintln!("Error: {err}"))
  .run(
    &host_from_args(),
    HttpRouter::new(wtx::paths!(("/hello", get(hello))), CorsMiddleware::permissive())?,
  )
  .await
}

async fn hello() -> &'static str {
  "Hello"
}
