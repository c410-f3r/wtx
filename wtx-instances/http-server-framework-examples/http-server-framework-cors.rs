//! The `CorsMiddleware` middleware inserts permissive CORS headers in every response.

use wtx::{
  http::server_framework::{CorsMiddleware, Router, ServerFrameworkBuilder, get},
  rng::{ChaCha20, CryptoSeedableRng},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let router = Router::new(wtx::paths!(("/hello", get(hello))), CorsMiddleware::permissive())?;
  ServerFrameworkBuilder::new(ChaCha20::from_getrandom()?, router)
    .without_aux()
    .tokio(
      "0.0.0.0:9000",
      |error: wtx::Error| eprintln!("{error:?}"),
      |_| Ok(()),
      |_| Ok(()),
      |error| eprintln!("{error}"),
    )
    .await?;
  Ok(())
}

async fn hello() -> &'static str {
  "Hello"
}
