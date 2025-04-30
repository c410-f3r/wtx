//! WTX instances

#![allow(
  clippy::allow_attributes_without_reason,
  clippy::arithmetic_side_effects,
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

#[cfg(any(feature = "mysql", feature = "postgres"))]
use {tokio::net::TcpStream, wtx::misc::Uri};

/// Certificate
pub static CERT: &[u8] = include_bytes!("../../.certs/cert.pem");
/// Private key
pub static KEY: &[u8] = include_bytes!("../../.certs/key.pem");
/// Root CA
pub static ROOT_CA: &[u8] = include_bytes!("../../.certs/root-ca.crt");

#[cfg(feature = "mysql")]
#[inline]
pub async fn executor_mysql(
  uri_str: &str,
) -> wtx::Result<
  wtx::database::client::mysql::MysqlExecutor<
    wtx::Error,
    wtx::database::client::mysql::ExecutorBuffer,
    TcpStream,
  >,
> {
  let uri = Uri::new(uri_str);
  let mut rng = wtx::rng::Xorshift64::from(wtx::rng::simple_seed());
  wtx::database::client::mysql::MysqlExecutor::connect(
    &wtx::database::client::mysql::Config::from_uri(&uri)?,
    wtx::database::client::mysql::ExecutorBuffer::new(usize::MAX, &mut rng),
    TcpStream::connect(uri.hostname_with_implied_port()).await?,
  )
  .await
}

#[cfg(feature = "postgres")]
#[inline]
pub async fn executor_postgres(
  uri_str: &str,
) -> wtx::Result<
  wtx::database::client::postgres::PostgresExecutor<
    wtx::Error,
    wtx::database::client::postgres::ExecutorBuffer,
    TcpStream,
  >,
> {
  use rand_chacha::{ChaCha20Rng, rand_core::SeedableRng};

  let uri = Uri::new(uri_str);
  let mut rng = ChaCha20Rng::try_from_os_rng()?;
  wtx::database::client::postgres::PostgresExecutor::connect(
    &wtx::database::client::postgres::Config::from_uri(&uri)?,
    wtx::database::client::postgres::ExecutorBuffer::new(usize::MAX, &mut rng),
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
