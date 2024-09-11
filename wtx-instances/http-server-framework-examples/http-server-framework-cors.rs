//! The `CorsMiddleware` middleware inserts permissive CORS headers in every response.

use wtx::http::server_framework::{get, CorsMiddleware, Router, ServerFrameworkBuilder};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let router = Router::new(wtx::paths!(("/hello", get(hello))), (), CorsMiddleware::permissive())?;
  ServerFrameworkBuilder::new(router)
    .without_aux()
    .listen("0.0.0.0:9000", |error: wtx::Error| eprintln!("{error:?}"))
    .await?;
  Ok(())
}

async fn hello() -> &'static str {
  "Hello"
}
