use crate::{
  crypto::{Aead, Hash},
  tls::CipherSuiteTy,
};

/// Defines the pair of the AEAD algorithm and hash algorithm.
pub trait CipherSuite {
  /// Authenticated encryption with associated data
  type Aead: Aead;
  /// See [`Hash`].
  type Hash: Hash;

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
