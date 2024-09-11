//! Different types of redirects.

use wtx::http::{
  server_framework::{get, Redirect, Router, ServerFrameworkBuilder, StateClean},
  ReqResBuffer, StatusCode,
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let router =
    Router::paths(wtx::paths!(("/permanent", get(permanent)), ("/temporary", get(temporary))))?;
  ServerFrameworkBuilder::new(router)
    .without_aux()
    .listen(&wtx_instances::host_from_args(), |error| eprintln!("{error}"))
    .await
}

async fn permanent() -> Redirect {
  Redirect::permanent("/some/path")
}

async fn temporary(state: StateClean<'_, (), (), ReqResBuffer>) -> wtx::Result<StatusCode> {
  Redirect::temporary_raw(&mut state.req.rrd.headers, "/another/path")
}
