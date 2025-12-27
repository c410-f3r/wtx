use crate::{misc::UriString, sync::AtomicU32};
use alloc::string::String;
use core::sync::atomic::Ordering;
use std::sync::OnceLock;
use wtx::misc::EnvVars;

#[allow(unused, reason = "depends on feature")]
#[derive(Debug, wtx::FromVars)]
pub(crate) struct _Vars {
  pub(crate) database_uri_mysql: String,
  pub(crate) database_uri_postgres: String,
}

pub(crate) fn _uri() -> UriString {
  const INITIAL_PORT: u32 = 7000;
  #[cfg(all(feature = "loom", not(feature = "portable-atomic")))]
  let port = {
    static LOCKS: OnceLock<AtomicU32> = std::sync::OnceLock::new();
    &*LOCKS.get_or_init(|| AtomicU32::new(INITIAL_PORT))
  };
  #[cfg(any(not(feature = "loom"), feature = "portable-atomic"))]
  let port = {
    static PORT: AtomicU32 = AtomicU32::new(INITIAL_PORT);
    &PORT
  };
  let uri = alloc::format!("http://127.0.0.1:{}", port.fetch_add(1, Ordering::Relaxed));
  UriString::new(uri)
}

pub(crate) fn _vars() -> &'static _Vars {
  static VARS: OnceLock<_Vars> = OnceLock::new();
  VARS.get_or_init(|| EnvVars::from_available().unwrap().finish())
}
