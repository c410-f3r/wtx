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

pub mod grpc_bindings;

/// Certificate
pub static CERT: &[u8] = include_bytes!("../../.certs/cert.pem");
/// Private key
pub static KEY: &[u8] = include_bytes!("../../.certs/key.pem");
/// Root CA
pub static ROOT_CA: &[u8] = include_bytes!("../../.certs/root-ca.crt");

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