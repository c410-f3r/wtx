//! Encryption algorithms negotiated at the handshake level

mod aes_128_gcm_sha_256;
mod aes_256_gcm_sha_384;
mod chacha20_poly1305_sha256;
mod cipher_suite_impl;
mod cipher_suite_ty;
mod wrappers;

pub use cipher_suite_ty::CipherSuiteTy;
pub use wrappers::*;

/// Defines the pair of the AEAD algorithm and hash algorithm to be used with HKDF.
pub trait CipherSuite {
  /// See [CipherSuiteTy].
  const TY: CipherSuiteTy;

  /// Authenticated encryption with associated data
  type Aead;
  /// Maps data of arbitrary size into a fixed-size value.
  type Hash;
}
