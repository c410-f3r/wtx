//! Different types of redirects.

use wtx::{
  http::{
    StatusCode,
    http2_server_framework::{Http2ServerFramework, HttpRouter, Redirect, StateClean, get},
  },
  rng::{ChaCha20, CryptoSeedableRng as _},
  tls::{TlsConfig, TlsModeVerified},
};
use wtx_examples::{PUBLIC_KEY, SECRET_KEY, host_from_args};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  Http2ServerFramework::tokio(
    ChaCha20::from_getrandom()?,
    TlsConfig::from_keys_pem(
      TlsModeVerified::default(),
      PUBLIC_KEY.try_into()?,
      SECRET_KEY.try_into()?,
    )?,
  )?
  .run(
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
