//! The `CorsMiddleware` middleware inserts permissive CORS headers in every response.

use wtx::{
  http::http2_server_framework::{CorsMiddleware, Http2ServerFramework, HttpRouter, get},
  tls::TlsConfig,
};
use wtx_examples::{PUBLIC_KEY, SECRET_KEY, host_from_args};

fn main() -> wtx::Result<()> {
  Http2ServerFramework::tokio(TlsConfig::from_keys_pem(PUBLIC_KEY, SECRET_KEY)?)?
    .set_error_cb(|err| eprintln!("Error: {err}"))
    .set_http_router(HttpRouter::new(wtx::paths!(("/hello", get(hello))), CorsMiddleware::new())?)
    .run_in_threads(&host_from_args())
}

async fn hello() -> &'static str {
  "Hello"
}
