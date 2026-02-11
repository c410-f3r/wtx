//! Algorithms that prevent third parties or the public from reading private messages.
//!
//! The structures available in this module are intended for internal operations but they can be
//! useful for public utilization.

#[macro_use]
mod macros;

mod aead;
mod agreement;
mod agreement_algorithm_ty;
mod crypto_error;
mod hash;
mod hkdf;

pub use aead::Aead;
pub use agreement::Agreement;
pub use agreement_algorithm_ty::AgreementAlgorithmTy;
pub use crypto_error::CryptoError;
pub use hash::Hash;
pub use hkdf::Hkdf;

/// Maximum hash length
//
// Based on Sha386.
pub const MAX_HASH_LEN: usize = 48;
/// Maximum public key length
//
// Based on P-384 uncompressed.
pub const MAX_PK_LEN: usize = 97;

#[cfg(feature = "aes-gcm")]
_create_wrapper!((Aes128GcmRustCrypto), (Aes256GcmRustCrypto));

#[cfg(feature = "aws-lc-rs")]
_create_wrapper!(
  (Aes128GcmAwsLcRs),
  (Aes256GcmAwsLcRs),
  (Chacha20Poly1305AwsLcRs),
  //
  (P256AwsLcRs),
  (P384AwsLcRs),
  (X25519AwsLcRs),
  //
  (Sha256DigestAwsLcRs),
  (Sha384DigestAwsLcRs),
  //
  (Sha256HkdfAwsLcRs, aws_lc_rs::hkdf::Prk),
  (Sha384HkdfAwsLcRs, aws_lc_rs::hkdf::Prk),
);

#[cfg(feature = "chacha20poly1305")]
_create_wrapper!((Chacha20Poly1305RustCrypto));
