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
  Aead, AeadDummy,
  global::{Aes128GcmGlobal, Aes256GcmGlobal, Chacha20Poly1305TyGlobal},
};
pub use agreement::{
  Agreement, AgreementDummy,
  global::{P256AgreementGlobal, P384AgreementGlobal, X25519Global},
};
pub use crypto_error::CryptoError;
pub use hash::{
  Hash, HashDummy,
  global::{Sha1DigestGlobal, Sha256DigestGlobal, Sha386DigestGlobal},
};
pub use hkdf::{
  Hkdf, HkdfDummy,
  global::{HkdfSha256Global, HkdfSha384Global},
};
pub use hmac::{
  Hmac, HmacDummy,
  global::{HmacSha256Global, HmacSha384Global},
};
pub use sign_key::{SignKey, SignKeyDummy};
pub use signature::{
  Signature, SignatureDummy,
  global::{
    Ed25519Global, P256SignatureGlobal, P384SignatureGlobal, RsaPssRsaeSha256Global,
    RsaPssRsaeSha384Global,
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
#[cfg(feature = "crypto-graviola")]
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

#[cfg(feature = "crypto-openssl")]
_create_wrappers!(
  #[derive(Default)]
  Aes128GcmOpenssl<>(),
  #[derive(Default)]
  Aes256GcmOpenssl<>(),
  #[derive(Default)]
  Chacha20Poly1305Openssl<>(),
  //
  #[derive(Default)]
  P256Openssl<>(),
  #[derive(Default)]
  P384Openssl<>(),
  #[derive(Default)]
  X25519Openssl<>(),
  //
  #[derive(Default)]
  Sha1DigestOpenssl<>(),
  #[derive(Default)]
  Sha256DigestOpenssl<>(),
  #[derive(Default)]
  Sha384DigestOpenssl<>(),
  //
  //
  HmacSha256Openssl<>(HmacOpenssl),
  HmacSha384Openssl<>(HmacOpenssl),
  //
  Ed25519SignKeyOpenssl<>(openssl::pkey::PKey<openssl::pkey::Private>),
  P256SignKeyOpenssl<>(openssl::pkey::PKey<openssl::pkey::Private>),
  P384SignKeyOpenssl<>(openssl::pkey::PKey<openssl::pkey::Private>),
  RsaPssSignKeySha256Openssl<>(openssl::pkey::PKey<openssl::pkey::Private>),
  RsaPssSignKeySha384Openssl<>(openssl::pkey::PKey<openssl::pkey::Private>),
  //
  #[derive(Default)]
  Ed25519Openssl<>(),
  #[derive(Default)]
  RsaPssRsaeSha256Openssl<>(),
  #[derive(Default)]
  RsaPssRsaeSha384Openssl<>(),
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

/// HMAC helper for OpenSSL
#[cfg(feature = "crypto-openssl")]
pub struct HmacOpenssl {
  signer: openssl::sign::Signer<'static>,
}

#[cfg(feature = "crypto-openssl")]
impl HmacOpenssl {
  fn new(digest: openssl::hash::MessageDigest, key: &[u8]) -> crate::Result<Self> {
    let pkey = openssl::pkey::PKey::hmac(key)?;
    let signer = openssl::sign::Signer::new(digest, &pkey)?;
    Ok(Self { signer })
  }
}

#[cfg(feature = "crypto-openssl")]
impl core::fmt::Debug for HmacOpenssl {
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("HmacOpenssl").finish()
  }
}

#[allow(clippy::panic, reason = "dummy structures should not be called")]
fn dummy_impl_call() -> ! {
  panic!(
    "An operation required a crypto algorithm but no crypto backend was selected! You can, for example, enable the `crypto-ring` feature."
  );
}
