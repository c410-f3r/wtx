//! Different types of redirects.

use wtx::{
  executor::TokioExecutor,
  http::{
    StatusCode,
    http2_server_framework::{Http2ServerFramework, HttpRouter, Redirect, StateClean, get},
  },
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
  let router =
    HttpRouter::paths(wtx::paths!(("/permanent", get(permanent)), ("/temporary", get(temporary))))?;
  Http2ServerFramework::new(TokioExecutor::default(), rng, tls_config)?
    .set_error_cb(|err| eprintln!("Error: {err}"))
    .run_in_threads(&host_from_args(), router)
}

async fn permanent() -> Redirect {
  Redirect::permanent("/some/path")
}

async fn temporary(state: StateClean<'_, ()>) -> wtx::Result<StatusCode> {
  Redirect::temporary_raw(&mut state.req.msg_data.headers, "/another/path")
}
