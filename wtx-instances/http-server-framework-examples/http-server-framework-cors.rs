//! The `CorsMiddleware` middleware inserts permissive CORS headers in every response.

use wtx::{
  http::server_framework::{get, CorsMiddleware, Router, ServerFrameworkBuilder},
  misc::{simple_seed, Xorshift64},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let router = Router::new(wtx::paths!(("/hello", get(hello))), CorsMiddleware::permissive())?;
  ServerFrameworkBuilder::new(router)
    .without_aux()
    .tokio(
      "0.0.0.0:9000",
      Xorshift64::from(simple_seed()),
      |error: wtx::Error| eprintln!("{error:?}"),
      |_| Ok(()),
    )
    .await?;
  Ok(())
}

async fn hello() -> &'static str {
  "Hello"
}
