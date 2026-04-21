//! `EnvVars` allows the interactive reading of environment variables.

extern crate wtx;

use std::sync::OnceLock;
use wtx::{
  calendar::{DateTime, Utc},
  collection::Vector,
  misc::EnvVars,
};

static VARS: OnceLock<Vars> = OnceLock::new();

fn main() -> wtx::Result<()> {
  let _rslt = VARS.set(EnvVars::from_available([])?.finish());
  let Vars { now, origin, port, root_ca, rust_log } = VARS.wait();
  println!(
    "`NOW={now:?}`, `ORIGIN={origin}`, `PORT={port}`, `ROOT_CA={root_ca:?}` and `RUST_LOG={rust_log:?}`"
  );
  Ok(())
}

#[derive(Debug, wtx::FromVars)]
struct Vars {
  #[from_vars(map_now)]
  now: Option<DateTime<Utc>>,
  origin: String,
  #[from_vars(map_port)]
  port: u16,
  root_ca: Vector<u8>,
  rust_log: Option<String>,
}

fn map_now(var: String) -> wtx::Result<DateTime<Utc>> {
  DateTime::from_iso8601(var.as_bytes())
}

fn map_port(var: String) -> wtx::Result<u16> {
  Ok(var.parse()?)
}
