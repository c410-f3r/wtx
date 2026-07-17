//! The `CorsMiddleware` middleware inserts permissive CORS headers in every response.

use wtx::{
  executor::TokioExecutor,
  http::http2_server_framework::{CorsMiddleware, Http2ServerFramework, HttpRouter, get},
  misc::SecretContext,
  rng::{ChaCha20, CryptoSeedableRng as _},
  tls::{TlsConfig, TlsModeVerified},
};
use wtx_examples::{PUBLIC_KEY, SECRET_KEY, host_from_args};

fn main() -> wtx::Result<()> {
  let mut rng = ChaCha20::from_getrandom()?;
  let secret_context = SecretContext::new(&mut rng)?;
  let tls_config = TlsConfig::from_keys_pem(
    TlsModeVerified::default(),
    PUBLIC_KEY.try_into()?,
    &mut rng,
    (secret_context, &mut SECRET_KEY.clone()),
  )?;
  let router = HttpRouter::new(wtx::paths!(("/hello", get(hello))), CorsMiddleware::permissive())?;
  Http2ServerFramework::new(TokioExecutor::default(), rng, tls_config)?
    .set_error_cb(|err| eprintln!("Error: {err}"))
    .run_in_threads(&host_from_args(), router)
}

async fn hello() -> &'static str {
  "Hello"
}
