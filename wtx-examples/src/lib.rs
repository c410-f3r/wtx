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

/// Public key
pub static PUBLIC_KEY: &[u8] = include_bytes!("../../.certs/cert.pem");
/// Secret key
pub static SECRET_KEY: &[u8] = include_bytes!("../../.certs/key.pem");
/// Root CA
pub static ROOT_CA: &[u8] = include_bytes!("../../.certs/root-ca.crt");

/// Illustrates how a tls stream can be converted into a plain-text stream for testing purposes.
pub type LocalTlsMode = cfg_select! {
  feature = "prod" => wtx::tls::TlsModeVerified,
  _ => wtx::tls::TlsModePlainText,
};

/// Generic Postgres client
#[cfg(feature = "postgres")]
pub async fn postgres_client(
  uri_str: &str,
) -> wtx::Result<
  wtx::database::client::postgres::PostgresClient<
    wtx::database::client::postgres::ClientBuffer,
    wtx::Error,
    tokio::net::TcpStream,
  >,
> {
  use wtx::rng::CryptoSeedableRng;
  let uri = wtx::misc::Uri::new(uri_str);
  let mut rng = wtx::rng::ChaCha20::from_getrandom()?;
  wtx::database::client::postgres::PostgresClient::connect(
    wtx::database::client::postgres::ClientBuffer::new(usize::MAX, &mut rng),
    &wtx::database::client::postgres::Config::from_uri(&uri)?,
    &mut rng,
    tokio::net::TcpStream::connect(uri.hostname_with_implied_port()).await?,
    None,
  )
  .await
}

/// Host from arguments
pub fn host_from_args() -> String {
  std::env::args().nth(1).unwrap_or_else(|| "127.0.0.1:9000".to_owned())
}

/// Uri from arguments
pub fn uri_from_args() -> String {
  std::env::args().nth(1).unwrap_or_else(|| "http://127.0.0.1:9000".to_owned())
}
