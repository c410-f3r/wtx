//! The `CorsMiddleware` middleware inserts permissive CORS headers in every response.

use wtx::{
  executor::TokioExecutor,
  http::http2_server_framework::{CorsMiddleware, Http2ServerFramework, HttpRouter, get},
  tls::TlsConfig,
};
use wtx_examples::{LocalTlsMode, PUBLIC_KEY, SECRET_KEY, host_from_args};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  Http2ServerFramework::new(
    TokioExecutor::default(),
    TlsConfig::from_keys_pem(
      LocalTlsMode::default(),
      PUBLIC_KEY.try_into()?,
      SECRET_KEY.try_into()?,
    )?
    .into(),
  )?
  .run_in_threads(
    &host_from_args(),
    HttpRouter::new(wtx::paths!(("/hello", get(hello))), CorsMiddleware::permissive())?,
  )
  .await?;
  Ok(())
}

async fn hello() -> &'static str {
  "Hello"
}
