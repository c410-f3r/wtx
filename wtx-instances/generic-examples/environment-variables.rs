//! `EnvVars` allows the interactive reading of environment variables.

extern crate wtx;
extern crate wtx_macros;

use std::sync::OnceLock;
use wtx::{
  calendar::{DateTime, Utc},
  misc::EnvVars,
};

static VARS: OnceLock<Vars> = OnceLock::new();

fn main() -> wtx::Result<()> {
  let _rslt = VARS.set(EnvVars::from_available()?.finish());
  let Vars { now, origin, port, rust_log } = VARS.wait();
  println!("`NOW={now:?}`, `ORIGIN={origin}`, `PORT={port}` and `RUST_LOG={rust_log:?}`");
  Ok(())
}

#[derive(Debug, wtx_macros::FromVars)]
struct Vars {
  #[from_vars(map_now)]
  now: Option<DateTime<Utc>>,
  origin: String,
  #[from_vars(map_port)]
  port: u16,
  rust_log: Option<String>,
}

fn map_now(var: String) -> wtx::Result<DateTime<Utc>> {
  Ok(DateTime::from_iso8601(var.as_bytes())?)
}

fn map_port(var: String) -> wtx::Result<u16> {
  Ok(var.parse()?)
}
