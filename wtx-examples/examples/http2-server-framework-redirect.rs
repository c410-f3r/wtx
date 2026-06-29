//! Different types of redirects.

use wtx::{
  executor::TokioExecutor,
  http::{
    StatusCode,
    http2_server_framework::{Http2ServerFramework, HttpRouter, Redirect, StateClean, get},
  },
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
    HttpRouter::paths(wtx::paths!(("/permanent", get(permanent)), ("/temporary", get(temporary))))?,
  )
  .await
}

async fn permanent() -> Redirect {
  Redirect::permanent("/some/path")
}

async fn temporary(state: StateClean<'_, ()>) -> wtx::Result<StatusCode> {
  Redirect::temporary_raw(&mut state.req.msg_data.headers, "/another/path")
}
