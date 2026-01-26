use crate::tls::{CipherSuiteTy, hash::Hash, hkdf::Hkdf};

/// Defines the pair of the AEAD algorithm and hash algorithm.
pub trait CipherSuite {
  /// Authenticated encryption with associated data
  type Aead;
  /// See [`Hash`].
  type Hash: Hash;
  /// See [`Hkdf`].
  //
  // This type is here because Rust Crypto needs to know the hash type
  type Hkdf: Hkdf;

  /// See [CipherSuiteTy].
  fn ty(&self) -> CipherSuiteTy;
}

impl CipherSuite for () {
  type Aead = ();
  type Hash = ();
  type Hkdf = ();

  fn ty(&self) -> CipherSuiteTy {
    CipherSuiteTy::Aes128GcmSha256
  }
}
