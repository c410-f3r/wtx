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
mod signature;

pub use aead::{
  Aead, AeadStub,
  global::{GlobalAes128GcmTy, GlobalAes256GcmTy, GlobalChacha20Poly1305Ty},
};
pub use agreement::{
  Agreement, AgreementStub,
  global::{GlobalP256Agreement, GlobalP384Agreement, GlobalX25519},
};
pub use crypto_error::CryptoError;
pub use hash::{
  Hash, HashStub,
  global::{GlobalSha1, GlobalSha256, GlobalSha386},
};
pub use hkdf::{
  Hkdf, HkdfStub,
  global::{GlobalHkdfSha256, GlobalHkdfSha384},
};
pub use signature::{
  Signature, SignatureStub,
  global::{GlobalEd25519, GlobalP256Signature, GlobalP384Signature},
  signature_ty::SignatureTy,
};

/// Maximum hash length
//
// Based on Sha386.
pub const MAX_HASH_LEN: usize = 48;
/// Maximum public key length
//
// Based on P-384 uncompressed.
pub const MAX_PK_LEN: usize = 97;

/// A wrapper around public keys or other external structures that don't implement `AsRef<[u8]>`.
#[cfg(any(feature = "crypto-graviola", feature = "p256", feature = "p384"))]
#[derive(Debug)]
pub struct AsRefWrapper<T>(T);

#[cfg(feature = "crypto-aws-lc-rs")]
_create_wrappers!(
  #[derive(Default)]
  Aes128GcmAwsLcRs<>(),
  #[derive(Default)]
  Aes256GcmAwsLcRs<>(),
  #[derive(Default)]
  Chacha20Poly1305AwsLcRs<>(),
  //
  #[derive(Default)]
  P256AwsLcRs<>(),
  #[derive(Default)]
  P384AwsLcRs<>(),
  #[derive(Default)]
  X25519AwsLcRs<>(),
  //
  #[derive(Default)]
  Sha1DigestAwsLcRs<>(),
  #[derive(Default)]
  Sha256DigestAwsLcRs<>(),
  #[derive(Default)]
  Sha384DigestAwsLcRs<>(),
  //
  HkdfSha256AwsLcRs<>(aws_lc_rs::hkdf::Prk),
  HkdfSha384AwsLcRs<>(aws_lc_rs::hkdf::Prk),
  //
  #[derive(Default)]
  Ed25519AwsLcRs<>(),
);

#[cfg(feature = "crypto-graviola")]
_create_wrappers!(
  #[derive(Default)]
  Aes128GcmGraviola<>(),
  #[derive(Default)]
  Aes256GcmGraviola<>(),
  #[derive(Default)]
  Chacha20Poly1305Graviola<>(),
  //
  #[derive(Default)]
  P256Graviola<>(),
  #[derive(Default)]
  P384Graviola<>(),
  #[derive(Default)]
  X25519Graviola<>(),
  //
  #[derive(Default)]
  Sha256DigestGraviola<>(),
  #[derive(Default)]
  Sha384DigestGraviola<>(),
  //
  #[derive(Default)]
  HkdfSha256Graviola<>(),
  #[derive(Default)]
  HkdfSha384Graviola<>(),
  //
  #[derive(Default)]
  Ed25519Graviola<>(),
);

#[cfg(feature = "crypto-ring")]
_create_wrappers!(
  #[derive(Default)]
  Aes128GcmRing<>(),
  #[derive(Default)]
  Aes256GcmRing<>(),
  #[derive(Default)]
  Chacha20Poly1305Ring<>(),
  //
  #[derive(Default)]
  P256Ring<>(),
  #[derive(Default)]
  P384Ring<>(),
  #[derive(Default)]
  X25519Ring<>(),
  //
  #[derive(Default)]
  Sha1DigestRing<>(),
  #[derive(Default)]
  Sha256DigestRing<>(),
  #[derive(Default)]
  Sha384DigestRing<>(),
  //
  HkdfSha256Ring<>(ring::hkdf::Prk),
  HkdfSha384Ring<>(ring::hkdf::Prk),
  //
  #[derive(Default)]
  Ed25519Ring<>(),
);

// Rust Crypto

#[cfg(feature = "aes-gcm")]
_create_wrappers!(
  #[derive(Default)]
  Aes128GcmRustCrypto<>(),
  #[derive(Default)]
  Aes256GcmRustCrypto<>()
);
#[cfg(feature = "chacha20poly1305")]
_create_wrappers!(
  #[derive(Default)]
  Chacha20Poly1305RustCrypto<>()
);
#[cfg(feature = "ed25519-dalek")]
_create_wrappers!(
  #[derive(Default)]
  Ed25519RustCrypto<>()
);

#[cfg(feature = "hkdf")]
_create_wrappers!(
  HkdfSha256RustCrypto<>(::hkdf::Hkdf<sha2::Sha256>),
  HkdfSha384RustCrypto<>(::hkdf::Hkdf<sha2::Sha384>)
);
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
#[cfg(feature = "sha1")]
_create_wrappers!(
  #[derive(Default)]
  Sha1DigestRustCrypto<>()
);
#[cfg(feature = "sha2")]
_create_wrappers!(
  #[derive(Default)]
  Sha256DigestRustCrypto<>(),
  #[derive(Default)]
  Sha384DigestRustCrypto<>()
);
#[cfg(feature = "x25519-dalek")]
_create_wrappers!(
  #[derive(Default)]
  X25519RustCrypto<>()
);
