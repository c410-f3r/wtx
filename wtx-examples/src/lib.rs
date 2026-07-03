//! WTX instances

#[allow(
  clippy::allow_attributes_without_reason,
  clippy::arithmetic_side_effects,
  clippy::as_conversions,
  clippy::cast_lossless,
  clippy::elidable_lifetime_names,
  clippy::inline_trait_bounds,
  clippy::min_ident_chars,
  clippy::missing_inline_in_public_items,
  clippy::mod_module_files,
  clippy::shadow_reuse,
  clippy::shadow_unrelated,
  clippy::single_char_lifetime_names,
  clippy::std_instead_of_alloc,
  clippy::use_debug,
  clippy::wildcard_imports,
  missing_docs,
  single_use_lifetimes,
  reason = "generated code"
)]
#[cfg(feature = "grpc")]
pub mod grpc_bindings;

/// Public key
pub static PUBLIC_KEY: &[u8] = include_bytes!("../../.certs/cert.pem");
/// Secret key
pub static SECRET_KEY: &[u8] = include_bytes!("../../.certs/key.pem");
/// Root CA
pub static ROOT_CA: &[u8] = include_bytes!("../../.certs/root-ca.crt");

/// Generic Postgres client
#[cfg(feature = "postgres")]
#[inline]
pub async fn postgres_client(
  uri_str: &str,
) -> wtx::Result<
  wtx::database::client::postgres::PostgresClient<
    wtx::Error,
    tokio::net::TcpStream,
    wtx::tls::TlsModeVerified,
  >,
> {
  use tokio::net::TcpStream;
  use wtx::{
    database::client::postgres::{ClientBuffer, Config, PostgresClient},
    rng::{ChaCha20, CryptoSeedableRng as _},
    tls::{TlsConfig, TlsConnector, TlsModeVerified},
  };
  let uri = wtx::misc::Uri::new(uri_str);
  let mut tls_connector = TlsConnector::new(
    TlsConfig::from_trust_anchors_pem(TlsModeVerified::default(), [ROOT_CA])?,
    ChaCha20::from_getrandom()?,
    TcpStream::connect(uri.hostname_with_implied_port()).await?,
  );
  PostgresClient::connect(
    ClientBuffer::new(usize::MAX, tls_connector.rng_mut()),
    &Config::from_uri(&uri)?,
    tls_connector,
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
