//! h2load

use wtx::{
  http::{
    HttpRecvParams,
    http2_server_framework::{Http2ServerFramework, HttpRouter, get},
  },
  tls::TlsConfig,
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  Http2ServerFramework::tokio(TlsConfig::plaintext())?
    .set_http_recv_params(HttpRecvParams::with_permissive_params())
    .set_http_router(HttpRouter::from_paths(wtx::paths!(("/", get(root)),))?)
    .set_error_cb(|err| println!("Error: {err}"))
    .run("127.0.0.1:9000")
    .await
}

async fn root() -> wtx::Result<()> {
  Ok(())
}
