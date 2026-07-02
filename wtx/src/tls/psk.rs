use crate::{
  collections::ArrayVectorCopy,
  crypto::MAX_HASH_LEN,
  tls::{CipherSuite, PskTy},
};

/// Pre Shared Key
///
/// Used to resume previous sessions.
#[derive(Clone, Copy, Debug)]
pub struct Psk {
  /// See [`CipherSuite`]
  pub cipher_suite: CipherSuite,
  /// Data
  pub data: ArrayVectorCopy<u8, MAX_HASH_LEN>,
  /// See [`PskTy`]
  pub ty: PskTy,
}

impl Psk {
  /// Shortcut
  #[inline]
  pub fn new(
    cipher_suite: CipherSuite,
    data: ArrayVectorCopy<u8, MAX_HASH_LEN>,
    ty: PskTy,
  ) -> Self {
    Self { cipher_suite, data, ty }
  }
}
