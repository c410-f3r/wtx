//! The `CorsMiddleware` middleware inserts permissive CORS headers in every response.

use wtx::{
  http::server_framework::{CorsMiddleware, Router, ServerFrameworkBuilder, get},
  rng::{Xorshift64, simple_seed},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let router = Router::new(wtx::paths!(("/hello", get(hello))), CorsMiddleware::permissive()?)?;
  ServerFrameworkBuilder::new(Xorshift64::from(simple_seed()), router)
    .without_aux()
    .tokio(
      "0.0.0.0:9000",
      |error: wtx::Error| eprintln!("{error:?}"),
      |_| Ok(()),
      |error| eprintln!("{error}"),
    )
    .await?;
  Ok(())
}

async fn hello() -> &'static str {
  "Hello"
}
