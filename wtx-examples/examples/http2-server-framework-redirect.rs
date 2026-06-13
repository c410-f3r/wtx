//! Different types of redirects.

use wtx::{
  executor::TokioExecutor,
  http::{
    StatusCode,
    http2_server_framework::{Http2ServerFramework, HttpRouter, Redirect, StateClean, get},
  },
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  Http2ServerFramework::new(TokioExecutor)?
    .run_in_threads(
      &wtx_examples::host_from_args(),
      HttpRouter::paths(wtx::paths!(
        ("/permanent", get(permanent)),
        ("/temporary", get(temporary))
      ))?,
    )
    .await
}

async fn permanent() -> Redirect {
  Redirect::permanent("/some/path")
}

async fn temporary(state: StateClean<'_, ()>) -> wtx::Result<StatusCode> {
  Redirect::temporary_raw(&mut state.req.msg_data.headers, "/another/path")
}
