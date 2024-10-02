//! WTX instances

#![allow(
  clippy::allow_attributes_without_reason,
  clippy::arithmetic_side_effects,
  clippy::as_conversions,
  clippy::cast_lossless,
  clippy::missing_inline_in_public_items,
  clippy::mod_module_files,
  clippy::shadow_unrelated,
  clippy::single_char_lifetime_names,
  clippy::std_instead_of_alloc,
  clippy::use_debug,
  clippy::wildcard_imports,
  missing_docs
)]

#[cfg(feature = "grpc")]
pub mod grpc_bindings;

#[cfg(feature = "postgres")]
use {
  tokio::net::TcpStream,
  wtx::{
    database::client::postgres::{Config, Executor, ExecutorBuffer},
    misc::{simple_seed, Uri, Xorshift64},
  },
};

/// Certificate
pub static CERT: &[u8] = include_bytes!("../../.certs/cert.pem");
/// Private key
pub static KEY: &[u8] = include_bytes!("../../.certs/key.pem");
/// Root CA
pub static ROOT_CA: &[u8] = include_bytes!("../../.certs/root-ca.crt");

#[cfg(feature = "postgres")]
#[inline]
pub async fn executor_postgres(
  uri_str: &str,
) -> wtx::Result<Executor<wtx::Error, ExecutorBuffer, TcpStream>> {
  let uri = Uri::new(uri_str);
  let mut rng = Xorshift64::from(simple_seed());
  Executor::connect(
    &Config::from_uri(&uri)?,
    ExecutorBuffer::with_default_params(&mut rng)?,
    &mut rng,
    TcpStream::connect(uri.hostname_with_implied_port()).await?,
  )
  .await
}

/// Host from arguments
#[inline]
pub fn host_from_args() -> String {
  std::env::args().nth(1).unwrap_or_else(|| "127.0.0.1:9000".to_owned())
}

/// Uri from arguments
#[inline]
pub fn uri_from_args() -> String {
  std::env::args().nth(1).unwrap_or_else(|| "http://127.0.0.1:9000".to_owned())
}
