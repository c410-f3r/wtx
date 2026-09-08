//! `EnvVars` allows the interactive reading of environment variables.

extern crate wtx;

use std::sync::OnceLock;
use wtx::{
  calendar::{Datetime, Utc},
  collections::Vector,
  misc::EnvVars,
  secret::SecretStr,
};

static VARS: OnceLock<Vars> = OnceLock::new();

fn main() -> wtx::Result<()> {
  let http_secret = "Top secret information retrieved from a remote password vault";
  let others = [("HTTP_SECRET".into(), http_secret.into())];
  let _rslt = VARS.set(EnvVars::from_available(others)?.finish());
  let Vars { http_secret, now, port, root_ca, rust_log } = VARS.wait();
  println!("`NOW={now:?}`, `PORT={port}`, `ROOT_CA={root_ca:?}` and `RUST_LOG={rust_log:?}`");
  let _bytes = http_secret.peek()?;
  // Make API requests, decrypt AES, sign documents, do a flip, etc...
  Ok(())
}

#[derive(Debug, wtx::FromVars)]
struct Vars {
  http_secret: SecretStr,
  #[from_vars(map_now)]
  now: Option<Datetime<Utc>>,
  port: u16,
  root_ca: Vector<u8>,
  rust_log: Option<String>,
}

fn map_now(var: String) -> wtx::Result<Datetime<Utc>> {
  Datetime::from_iso8601(var.as_bytes())
}
