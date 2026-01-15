use crate::tls::protocol::ephemeral_secret_key::EphemeralSecretKey;

/// The back-end responsible for cryptography operations
pub(crate) trait TlsCrypto {
  type EphemeralSecretKey: EphemeralSecretKey;
}

impl TlsCrypto for () {
  type EphemeralSecretKey = ();
}

#[cfg(feature = "aws-lc-rs")]
#[derive(Debug, Default)]
pub struct AwsLcRs {}

#[cfg(feature = "aws-lc-rs")]
impl TlsCrypto for AwsLcRs {
  type EphemeralSecretKey = crate::tls::protocol::named_group::aws_lc_rs::NamedGroupParamAwsLcRs;
}

#[cfg(feature = "rust-crypto")]
#[derive(Debug, Default)]
pub struct RustCrypto {}

#[cfg(feature = "rust-crypto")]
impl TlsCrypto for RustCrypto {
  type EphemeralSecretKey =
    crate::tls::protocol::named_group::rust_crypto::NamedGroupParamRustCrypto;
}
