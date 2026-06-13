//! The `CorsMiddleware` middleware inserts permissive CORS headers in every response.

use wtx::{
  executor::TokioExecutor,
  http::http2_server_framework::{CorsMiddleware, Http2ServerFramework, HttpRouter, get},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  Http2ServerFramework::new(TokioExecutor)?
    .run_in_threads(
      "0.0.0.0:9000",
      HttpRouter::new(wtx::paths!(("/hello", get(hello))), CorsMiddleware::permissive())?,
    )
    .await?;
  Ok(())
}

async fn hello() -> &'static str {
  "Hello"
}
