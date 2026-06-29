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
  global::{Aes128GcmGlobal, Aes256GcmGlobal, Chacha20Poly1305Global},
};
pub use agreement::{
  Agreement, AgreementDummy,
  global::{P256AgreementGlobal, P384AgreementGlobal, X25519Global},
};
pub use crypto_error::CryptoError;
pub use hash::{
  Hash, HashDummy,
  global::{Sha1HashGlobal, Sha256HashGlobal, Sha384HashGlobal},
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

use crate::rng::CryptoRng;

/// AEAD nonce prefix
pub const AEAD_NONCE_LEN: usize = 12;
/// AEAD tag suffix
pub const AEAD_TAG_LEN: usize = 16;
/// Maximum hash length
//
// Based on Sha384.
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
  P256AwsLcRs<>(aws_lc_rs::agreement::EphemeralPrivateKey),
  P384AwsLcRs<>(aws_lc_rs::agreement::EphemeralPrivateKey),
  #[derive(Default)]
  RsaPssRsaeSha256AwsLcRs<>(),
  #[derive(Default)]
  RsaPssRsaeSha384AwsLcRs<>(),
  X25519AwsLcRs<>(aws_lc_rs::agreement::EphemeralPrivateKey),
  //
  #[derive(Default)]
  Sha1HashAwsLcRs<>(),
  #[derive(Default)]
  Sha256HashAwsLcRs<>(),
  #[derive(Default)]
  Sha384HashAwsLcRs<>(),
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
  P256Graviola<>(graviola::key_agreement::p256::PrivateKey),
  P384Graviola<>(graviola::key_agreement::p384::PrivateKey),
  #[derive(Default)]
  RsaPssRsaeSha256Graviola<>(),
  #[derive(Default)]
  RsaPssRsaeSha384Graviola<>(),
  X25519Graviola<>(graviola::key_agreement::x25519::PrivateKey),
  //
  #[derive(Default)]
  Sha256HashGraviola<>(),
  #[derive(Default)]
  Sha384HashGraviola<>(),
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
  P256Openssl<>(openssl::pkey::PKey<openssl::pkey::Private>),
  P384Openssl<>(openssl::pkey::PKey<openssl::pkey::Private>),
  X25519Openssl<>(openssl::pkey::PKey<openssl::pkey::Private>),
  //
  #[derive(Default)]
  Sha1HashOpenssl<>(),
  #[derive(Default)]
  Sha256HashOpenssl<>(),
  #[derive(Default)]
  Sha384HashOpenssl<>(),
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
  P256Ring<>(ring::agreement::EphemeralPrivateKey),
  P384Ring<>(ring::agreement::EphemeralPrivateKey),
  #[derive(Default)]
  RsaPssRsaeSha256Ring<>(),
  #[derive(Default)]
  RsaPssRsaeSha384Ring<>(),
  X25519Ring<>(ring::agreement::EphemeralPrivateKey),
  //
  #[derive(Default)]
  Sha1HashRing<>(),
  #[derive(Default)]
  Sha256HashRing<>(),
  #[derive(Default)]
  Sha384HashRing<>(),
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

/// HMAC helper for `OpenSSL`
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

/// AEAD nonce prefix
#[inline]
pub fn gen_aead_nonce<RNG>(rng: &mut RNG) -> [u8; AEAD_NONCE_LEN]
where
  RNG: CryptoRng,
{
  let [a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, _, _, _, _] = rng.u8_16();
  [a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11]
}

#[expect(clippy::panic, reason = "dummy structures should not be called")]
fn dummy_impl_call() -> ! {
  panic!(
    "An operation required a crypto algorithm but no crypto backend was selected! You can, for example, enable the `crypto-ring` feature."
  );
}
