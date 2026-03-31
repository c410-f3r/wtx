//! Algorithms that prevent third parties or the public from reading private messages.
//!
//! The structures available in this module are intended for internal operations but they can be
//! useful for public utilization.

#[macro_use]
mod macros;

mod aead;
mod agreement;
mod crypto_error;
mod hash;
mod hkdf;

pub use aead::Aead;
pub use agreement::Agreement;
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

/// A wrapper around public keys or other external structures that don't implement `AsRef<[u8]>`.
#[cfg(any(feature = "p256", feature = "p384"))]
#[derive(Debug)]
pub struct AsRefWrapper<T>(T);

#[cfg(feature = "aws-lc-rs")]
_create_wrappers!(
  Aes128GcmAwsLcRs<>(),
  Aes256GcmAwsLcRs<>(),
  Chacha20Poly1305AwsLcRs<>(),
  //
  #[derive(Default)]
  P256AwsLcRs<>(),
  #[derive(Default)]
  P384AwsLcRs<>(),
  #[derive(Default)]
  X25519AwsLcRs<>(),
  //
  Sha256DigestAwsLcRs<>(),
  Sha384DigestAwsLcRs<>(),
  //
  Sha256HkdfAwsLcRs<>(aws_lc_rs::hkdf::Prk),
  Sha384HkdfAwsLcRs<>(aws_lc_rs::hkdf::Prk),
);

#[cfg(feature = "graviola")]
_create_wrappers!(
  Aes128GcmGraviola<>(),
  Aes256GcmGraviola<>(),
  Chacha20Poly1305Graviola<>(),
  //
  #[derive(Default)]
  P256Graviola<>(),
  #[derive(Default)]
  P384Graviola<>(),
  #[derive(Default)]
  X25519Graviola<>(),
  //
  Sha256DigestGraviola<>(),
  Sha384DigestGraviola<>(),
  //
  Sha256HkdfGraviola<>(),
  Sha384HkdfGraviola<>(),
);

#[cfg(feature = "ring")]
_create_wrappers!(
  Aes128GcmRing<>(),
  Aes256GcmRing<>(),
  Chacha20Poly1305Ring<>(),
  //
  #[derive(Default)]
  P256Ring<>(),
  #[derive(Default)]
  P384Ring<>(),
  #[derive(Default)]
  X25519Ring<>(),
  //
  Sha256DigestRing<>(),
  Sha384DigestRing<>(),
  //
  Sha256HkdfRing<>(ring::hkdf::Prk),
  Sha384HkdfRing<>(ring::hkdf::Prk),
);

// Rust Crypto

#[cfg(feature = "aes-gcm")]
_create_wrappers!(Aes128GcmRustCrypto<>(), Aes256GcmRustCrypto<>());
#[cfg(feature = "chacha20poly1305")]
_create_wrappers!(Chacha20Poly1305RustCrypto<>());
#[cfg(feature = "hkdf")]
_create_wrappers!(HkdfRustCrypto<H: hmac::EagerHash>(::hkdf::Hkdf<H>));
#[cfg(feature = "p256")]
_create_wrappers!(
  #[derive(Default)]
  P256RustCrypto<>()
);
#[cfg(feature = "p384")]
_create_wrappers!(
  #[derive(Default)]
  P384RustCrypto<>()
);
#[cfg(feature = "sha2")]
_create_wrappers!(Sha256DigestRustCrypto<>(), Sha384DigestRustCrypto<>());
#[cfg(feature = "x25519-dalek")]
_create_wrappers!(
  #[derive(Default)]
  X25519RustCrypto<>()
);
