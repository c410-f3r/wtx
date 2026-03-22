use crate::{misc::UriString, sync::AtomicU32};
use alloc::string::String;
use core::sync::atomic::Ordering;
use std::sync::OnceLock;
use wtx::misc::EnvVars;

#[allow(unused, reason = "depends on feature")]
#[derive(Debug, wtx::FromVars)]
pub(crate) struct Vars {
  pub(crate) database_uri_mysql: String,
  pub(crate) database_uri_postgres: String,
}

pub(crate) fn _uri() -> UriString {
  const INITIAL_PORT: u32 = 7000;
  let port = {
    static PORT: AtomicU32 = AtomicU32::new(INITIAL_PORT);
    &PORT
  };
  let uri = alloc::format!("http://127.0.0.1:{}", port.fetch_add(1, Ordering::Relaxed));
  UriString::new(uri)
}

pub(crate) fn _vars() -> &'static Vars {
  static VARS: OnceLock<Vars> = OnceLock::new();
  VARS.get_or_init(|| EnvVars::from_available([]).unwrap().finish())
}
