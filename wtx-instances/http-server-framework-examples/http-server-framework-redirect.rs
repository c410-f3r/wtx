//! Different types of redirects.

use wtx::{
  http::{
    ReqResBuffer, StatusCode,
    server_framework::{Redirect, Router, ServerFrameworkBuilder, StateClean, get},
  },
  rng::{Xorshift64, simple_seed},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let router =
    Router::paths(wtx::paths!(("/permanent", get(permanent)), ("/temporary", get(temporary))))?;
  ServerFrameworkBuilder::new(Xorshift64::from(simple_seed()), router)
    .without_aux()
    .tokio(
      &wtx_instances::host_from_args(),
      |error| eprintln!("{error}"),
      |_| Ok(()),
      |error| eprintln!("{error}"),
    )
    .await
}

async fn permanent() -> Redirect {
  Redirect::permanent("/some/path")
}

async fn temporary(state: StateClean<'_, (), (), ReqResBuffer>) -> wtx::Result<StatusCode> {
  Redirect::temporary_raw(&mut state.req.rrd.headers, "/another/path")
}
