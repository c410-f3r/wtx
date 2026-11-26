//! Different types of redirects.

use wtx::{
  http::{
    StatusCode,
    http2_server_framework::{Http2ServerFramework, HttpRouter, Redirect, StateClean, get},
  },
  tls::TlsConfig,
};
use wtx_examples::{PUBLIC_KEY, SECRET_KEY, host_from_args};

fn main() -> wtx::Result<()> {
  Http2ServerFramework::tokio(TlsConfig::from_keys_pem(PUBLIC_KEY, SECRET_KEY)?)?
    .set_error_cb(|err| eprintln!("Error: {err}"))
    .set_http_router(HttpRouter::from_paths(wtx::paths!(
      ("/permanent", get(permanent)),
      ("/temporary", get(temporary))
    ))?)
    .run_in_threads(&host_from_args())
}

async fn permanent() -> Redirect {
  Redirect::permanent("/some/path")
}

async fn temporary(StateClean { req, data: _ }: StateClean<'_, ()>) -> wtx::Result<StatusCode> {
  Redirect::temporary_raw(&mut req.msg_data.headers, "/another/path")
}
