use crate::{
  crypto::{Aead, AeadDummy, Hash, HashDummy},
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
  type Aead = AeadDummy<[u8; 0]>;
  type Hash = HashDummy<[u8; 0]>;

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
