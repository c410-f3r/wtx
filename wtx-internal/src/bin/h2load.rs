//! h2load

use wtx::{
  executor::TokioExecutor,
  http::{
    HttpRecvParams,
    http2_server_framework::{Http2ServerFramework, HttpRouter, State, get},
  },
  tls::TlsConfig,
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let router = HttpRouter::paths(wtx::paths!(("/", get(root)),))?;
  Http2ServerFramework::new(TokioExecutor::default(), TlsConfig::empty())?
    .set_http_recv_params(HttpRecvParams::with_permissive_params())
    .run("127.0.0.1:9000", router)
    .await
}

async fn root(_state: State<'_, ()>) -> wtx::Result<()> {
  Ok(())
}
