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
mod hmac;
mod sign_key;
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
pub use hmac::{
  Hmac, HmacStub,
  global::{GlobalHmacSha256, GlobalHmacSha384},
};
pub use sign_key::{SignKey, SignKeyStub};
pub use signature::{
  Signature, SignatureStub,
  global::{
    GlobalEd25519, GlobalP256Signature, GlobalP384Signature, GlobalRsaPssRsaeSha256,
    GlobalRsaPssRsaeSha384,
  },
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
  RsaPssRsaeSha256AwsLcRs<>(),
  #[derive(Default)]
  RsaPssRsaeSha384AwsLcRs<>(),
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
  HmacSha256AwsLcRs<>(aws_lc_rs::hmac::Context),
  HmacSha384AwsLcRs<>(aws_lc_rs::hmac::Context),
  //
  #[derive(Default)]
  Ed25519AwsLcRs<>(),
  //
  Ed25519SignKeyAwsLcRs<>(aws_lc_rs::signature::Ed25519KeyPair),
  P256SignKeyAwsLcRs<>(aws_lc_rs::signature::EcdsaKeyPair),
  P384SignKeyAwsLcRs<>(aws_lc_rs::signature::EcdsaKeyPair),
  RsaPssSignKeySha384AwsLcRs<>(aws_lc_rs::signature::RsaKeyPair),
  RsaPssSignKeySha256AwsLcRs<>(aws_lc_rs::signature::RsaKeyPair),
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
  RsaPssRsaeSha256Graviola<>(),
  #[derive(Default)]
  RsaPssRsaeSha384Graviola<>(),
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
  HmacSha256Graviola<>(graviola::hashing::hmac::Hmac<graviola::hashing::Sha256>),
  HmacSha384Graviola<>(graviola::hashing::hmac::Hmac<graviola::hashing::Sha384>),
  //
  #[derive(Default)]
  Ed25519Graviola<>(),
  //
  Ed25519SignKeyGraviola<>(graviola::signing::eddsa::Ed25519SigningKey),
  P256SignKeyGraviola<>(graviola::signing::ecdsa::SigningKey<graviola::signing::ecdsa::P256>),
  P384SignKeyGraviola<>(graviola::signing::ecdsa::SigningKey<graviola::signing::ecdsa::P384>),
  RsaPssSignKeySha384Graviola<>(graviola::signing::rsa::SigningKey),
  RsaPssSignKeySha256Graviola<>(graviola::signing::rsa::SigningKey),
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
  RsaPssRsaeSha256Ring<>(),
  #[derive(Default)]
  RsaPssRsaeSha384Ring<>(),
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
  HmacSha256Ring<>(ring::hmac::Context),
  HmacSha384Ring<>(ring::hmac::Context),
  //
  #[derive(Default)]
  Ed25519Ring<>(),
  //
  Ed25519SignKeyRing<>(ring::signature::Ed25519KeyPair),
  P256SignKeyRing<>(ring::signature::EcdsaKeyPair),
  P384SignKeyRing<>(ring::signature::EcdsaKeyPair),
  RsaPssSignKeySha384Ring<>(ring::signature::RsaKeyPair),
  RsaPssSignKeySha256Ring<>(ring::signature::RsaKeyPair),
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
  Ed25519RustCrypto<>(),
  Ed25519SignKeyRustCrypto<>(ed25519_dalek::SigningKey)
);
#[cfg(feature = "hkdf")]
_create_wrappers!(
  HkdfSha256RustCrypto<>(::hkdf::Hkdf<sha2::Sha256>),
  HkdfSha384RustCrypto<>(::hkdf::Hkdf<sha2::Sha384>)
);
#[cfg(feature = "hmac")]
_create_wrappers!(
  HmacSha256RustCrypto<>(::hmac::Hmac<sha2::Sha256>),
  HmacSha384RustCrypto<>(::hmac::Hmac<sha2::Sha384>)
);
#[cfg(feature = "p256")]
_create_wrappers!(
  #[derive(Default)]
  P256RustCrypto<>(),
  P256SignKeyRustCrypto<>(p256::ecdsa::SigningKey)
);
#[cfg(feature = "p384")]
_create_wrappers!(
  #[derive(Default)]
  P384RustCrypto<>(),
  P384SignKeyRustCrypto<>(p384::ecdsa::SigningKey)
);
#[cfg(feature = "rsa")]
_create_wrappers!(
  #[derive(Default)]
  RsaPssRsaeSha256RustCrypto<>(),
  #[derive(Default)]
  RsaPssRsaeSha384RustCrypto<>(),
  RsaPssSignKeySha256RustCrypto<>(rsa::pss::SigningKey<sha2::Sha256>),
  RsaPssSignKeySha384RustCrypto<>(rsa::pss::SigningKey<sha2::Sha384>)
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
