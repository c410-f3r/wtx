//! testssl

use wtx::{
  http::http2_server_framework::{Http2ServerFramework, HttpRouter, State, VerbatimParams, get},
  misc::SecretContext,
  rng::{ChaCha20, CryptoSeedableRng as _},
  tls::{TlsConfig, TlsModeVerified},
};

pub static FULL_CHAIN: &[u8] = include_bytes!("../../../.certs/fullchain.pem");
pub static SECRET_KEY: &[u8; 119] = include_bytes!("../../../.certs/key.pem");

fn main() -> wtx::Result<()> {
  let mut rng = ChaCha20::from_std_random().unwrap();
  let secret_context = SecretContext::new(&mut rng).unwrap();
  let tls_config = TlsConfig::from_keys_pem(
    TlsModeVerified::default(),
    FULL_CHAIN.try_into()?,
    &mut rng,
    (secret_context, &mut SECRET_KEY.clone()),
  )?;
  let router = HttpRouter::paths(wtx::paths!(("/", get(root)),))?;
  Http2ServerFramework::tokio(tls_config)?.run_in_threads("0.0.0.0:9000", router)
}

async fn root(_: State<'_, ()>) -> wtx::Result<VerbatimParams> {
  Ok(VerbatimParams::default())
}
