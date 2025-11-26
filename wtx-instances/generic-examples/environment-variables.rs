//! `EnvVars` allows the interactive reading of environment variables.

extern crate wtx;
extern crate wtx_macros;

use std::sync::OnceLock;
use wtx::misc::EnvVars;

static VARS: OnceLock<Vars> = OnceLock::new();

fn main() -> wtx::Result<()> {
  let _rslt = VARS.set(EnvVars::from_available()?.finish());
  let Vars { origin, rust_log } = VARS.wait();
  println!("`ORIGIN={origin}` and `RUST_LOG={rust_log}`");
  Ok(())
}

#[derive(Debug, wtx_macros::FromVars)]
struct Vars {
  origin: String,
  rust_log: String,
}
