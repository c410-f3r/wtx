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

const MAX_CIPHER_KEY_LEN: usize = 32;
const MAX_HASH_LEN: usize = 48;
// Maximum length of P-384 uncompressed.
const MAX_PK_LEN: usize = 97;

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

#[cfg(feature = "aes-gcm")]
_create_wrapper!((Aes128GcmAesGcm), (Aes256GcmAesGcm));
