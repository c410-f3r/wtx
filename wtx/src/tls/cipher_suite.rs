//! Encryption algorithms negotiated at the handshake level
mod aes_128_gcm_sha_256;
mod aes_256_gcm_sha_384;
mod chacha20_poly1305_sha256;
pub(crate) mod cipher_suite_param;
mod cipher_suite_ty;
mod wrappers;

use crate::tls::{hash::Hash, hkdf::Hkdf};
pub use cipher_suite_ty::CipherSuiteTy;
pub use wrappers::*;

/// Defines the pair of the AEAD algorithm and hash algorithm to be used with HKDF.
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
