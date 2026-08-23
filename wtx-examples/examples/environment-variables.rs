//! `EnvVars` allows the interactive reading of environment variables.

extern crate wtx;

use std::sync::OnceLock;
use wtx::{
  calendar::{DateTime, Utc},
  collections::Vector,
  misc::{EnvVars, Secret, SecretContext},
  rng::{ChaCha20, CryptoSeedableRng},
};

static VARS: OnceLock<Vars> = OnceLock::new();

fn main() -> wtx::Result<()> {
  let http_secret = "Top secret information retrieved from a remote password vault";
  let others = [("HTTP_SECRET".into(), http_secret.into())];
  let _rslt = VARS.set(EnvVars::from_available(others)?.finish());
  let Vars { http_secret, now, port, root_ca, rust_log } = VARS.wait();
  println!("`NOW={now:?}`, `PORT={port}`, `ROOT_CA={root_ca:?}` and `RUST_LOG={rust_log:?}`");
  let mut buffer = Vector::new();
  let _sp = http_secret.peek(&mut buffer)?;
  // Make API requests, decrypt AES, sign documents, do a flip, etc...
  Ok(())
}

#[derive(Debug, wtx::FromVars)]
struct Vars {
  #[from_vars(map_secret)]
  http_secret: Secret,
  #[from_vars(map_now)]
  now: Option<DateTime<Utc>>,
  port: u16,
  root_ca: Vector<u8>,
  rust_log: Option<String>,
}

fn map_now(var: String) -> wtx::Result<DateTime<Utc>> {
  DateTime::from_iso8601(var.as_bytes())
}

fn map_secret(var: String) -> wtx::Result<Secret> {
  let mut rng = ChaCha20::from_std_random()?;
  let secret_context = SecretContext::new(&mut rng)?;
  Secret::new(&mut var.into_bytes(), &mut rng, secret_context)
}
