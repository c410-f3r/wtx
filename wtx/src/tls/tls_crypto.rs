use crate::{
  crypto::{Agreement, Hash, Hkdf},
  tls::{NamedGroup, cipher_suite::CipherSuite},
};

/// The back-end responsible for cryptography operations
pub trait TlsCrypto {
  /// See [`Agreement`].
  //
  // Users can choose different agreements
  type Agreement: Agreement + From<NamedGroup>;
  /// See [`CipherSuite`].
  //
  // Users can choose different cipher suites
  type CipherSuite: CipherSuite;
  /// See [`Hkdf`].
  //
  // Users don't choose KDFs but it is still necessary to have multiple options due to the
  // hash selected in the cipher suite.
  type Hkdf: Hkdf<Digest = <<Self::CipherSuite as CipherSuite>::Hash as Hash>::Digest>;
}

impl TlsCrypto for () {
  type Agreement = crate::tls::protocol::named_group::NamedGroupParam<(), (), ()>;
  type CipherSuite = ();
  type Hkdf = ();
}

/// AWS-LC backend
#[cfg(feature = "tls-aws-lc-rs")]
#[derive(Debug, Default)]
pub struct AwsLcRs {}
#[cfg(feature = "tls-aws-lc-rs")]
impl TlsCrypto for AwsLcRs {
  type Agreement = crate::tls::protocol::named_group::NamedGroupParam<
    crate::crypto::P256AwsLcRs,
    crate::crypto::P384AwsLcRs,
    crate::crypto::X25519AwsLcRs,
  >;
  type CipherSuite = ();
  type Hkdf = ();
}

/// Ring backend
#[cfg(feature = "tls-ring")]
#[derive(Debug, Default)]
pub struct RustRing {}
#[cfg(feature = "tls-ring")]
impl TlsCrypto for RustRing {
  type Agreement = crate::tls::protocol::named_group::NamedGroupParam<
    crate::crypto::P256Ring,
    crate::crypto::P384Ring,
    crate::crypto::X25519Ring,
  >;
  type CipherSuite = ();
  type Hkdf = ();
}

/// Rust crypto backend
#[cfg(feature = "tls-rust-crypto")]
#[derive(Debug, Default)]
pub struct RustCrypto {}
#[cfg(feature = "tls-rust-crypto")]
impl TlsCrypto for RustCrypto {
  type Agreement = crate::tls::protocol::named_group::NamedGroupParam<
    crate::crypto::P256RustCrypto,
    crate::crypto::P384RustCrypto,
    crate::crypto::X25519RustCrypto,
  >;
  type CipherSuite = ();
  type Hkdf = ();
}
