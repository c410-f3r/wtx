use crate::tls::{cipher_suite::CipherSuite, ephemeral_secret_key::EphemeralSecretKey};

/// The back-end responsible for cryptography operations
pub(crate) trait TlsCrypto {
  /// See [`CipherSuite`].
  type CipherSuite: CipherSuite;
  /// See [`EphemeralSecretKey`].
  type EphemeralSecretKey: EphemeralSecretKey;
}

impl TlsCrypto for () {
  type CipherSuite = crate::tls::protocol::cipher_suite_param::CipherSuiteParamUnit;
  type EphemeralSecretKey = ();
}

/// AWS-LC backend
#[cfg(feature = "aws-lc-rs")]
#[derive(Debug, Default)]
pub struct AwsLcRs {}

#[cfg(feature = "aws-lc-rs")]
impl TlsCrypto for AwsLcRs {
  type CipherSuite = crate::tls::protocol::cipher_suite_param::CipherSuiteParamAwsLcRs;
  type EphemeralSecretKey = crate::tls::protocol::named_group::aws_lc_rs::NamedGroupEpkAwsLcRs;
}

/// Rust crypto backend
#[cfg(feature = "rust-crypto")]
#[derive(Debug, Default)]
pub struct RustCrypto {}

#[cfg(feature = "rust-crypto")]
impl TlsCrypto for RustCrypto {
  type CipherSuite = crate::tls::protocol::cipher_suite_param::CipherSuiteParamRustCrypto;
  type EphemeralSecretKey = crate::tls::protocol::named_group::rust_crypto::NamedGroupEpkRustCrypto;
}
