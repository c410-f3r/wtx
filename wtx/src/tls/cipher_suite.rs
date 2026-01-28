use crate::{
  crypto::Aead,
  tls::{CipherSuiteTy, hash::Hash, hkdf::Hkdf},
};

static mut NOTHING: () = ();

/// Defines the pair of the AEAD algorithm and hash algorithm.
pub trait CipherSuite {
  /// Authenticated encryption with associated data
  type Aead: Aead;
  /// See [`Hash`].
  type Hash: Hash;
  /// See [`Hkdf`].
  //
  // This type is here because Rust Crypto needs to know the hash type
  type Hkdf: Hkdf;

  /// See [`Aead`].
  fn aead(&self) -> &Self::Aead;

  /// See [`Hash`].
  fn hash(&self) -> &Self::Hash;

  /// See [CipherSuiteTy].
  fn ty(&self) -> CipherSuiteTy;
}

impl CipherSuite for () {
  type Aead = ();
  type Hash = ();
  type Hkdf = ();

  #[inline]
  fn aead(&self) -> &Self::Aead {
    self
  }

  #[inline]
  fn hash(&self) -> &Self::Hash {
    self
  }

  #[inline]
  fn ty(&self) -> CipherSuiteTy {
    CipherSuiteTy::Aes128GcmSha256
  }
}
