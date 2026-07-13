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

use crate::rng::CryptoRng;
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
  signature_ty::{SignatureSignKey, SignatureSignOutput, SignatureTy},
};

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
  #[derive(Clone)]
  Sha1HashAwsLcRs<>(aws_lc_rs::digest::Context),
  #[derive(Clone)]
  Sha256HashAwsLcRs<>(aws_lc_rs::digest::Context),
  #[derive(Clone)]
  Sha384HashAwsLcRs<>(aws_lc_rs::digest::Context),
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
  #[derive(Clone)]
  Sha256HashGraviola<>(<graviola::hashing::Sha256 as graviola::hashing::Hash>::Context),
  #[derive(Clone)]
  Sha384HashGraviola<>(<graviola::hashing::Sha384 as graviola::hashing::Hash>::Context),
  //
  HkdfSha256Graviola<>(GraviolaPrk<graviola::hashing::Sha256>),
  HkdfSha384Graviola<>(GraviolaPrk<graviola::hashing::Sha384>),
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
  P256Ring<>(ring::agreement::EphemeralPrivateKey),
  P384Ring<>(ring::agreement::EphemeralPrivateKey),
  #[derive(Default)]
  RsaPssRsaeSha256Ring<>(),
  #[derive(Default)]
  RsaPssRsaeSha384Ring<>(),
  X25519Ring<>(ring::agreement::EphemeralPrivateKey),
  //
  #[derive(Clone)]
  Sha1HashRing<>(ring::digest::Context),
  #[derive(Clone)]
  Sha256HashRing<>(ring::digest::Context),
  #[derive(Clone)]
  Sha384HashRing<>(ring::digest::Context),
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

/// AEAD nonce prefix
#[inline]
pub fn gen_aead_nonce<RNG>(rng: &mut RNG) -> [u8; AEAD_NONCE_LEN]
where
  RNG: CryptoRng,
{
  let [a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, _, _, _, _] = rng.u8_16();
  [a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11]
}

/// HDKF implementation for Graviola
#[cfg(feature = "crypto-graviola")]
#[derive(Debug)]
pub struct GraviolaPrk<H> {
  output: graviola::hashing::HashOutput,
  phantom: core::marker::PhantomData<H>,
}

#[cfg(feature = "crypto-graviola")]
impl<H> GraviolaPrk<H>
where
  H: Clone + graviola::hashing::Hash,
{
  #[inline]
  fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (graviola::hashing::HashOutput, GraviolaPrk<H>) {
    let mut hmac = match salt {
      Some(elem) => graviola::hashing::hmac::Hmac::<H>::new(elem),
      None => graviola::hashing::hmac::Hmac::<H>::new(H::zeroed_output()),
    };
    hmac.update(ikm);
    let output = hmac.finish();
    (output.clone(), Self { output, phantom: core::marker::PhantomData })
  }

  #[inline]
  fn new(slice: &[u8]) -> crate::Result<Self> {
    let mut output = H::zeroed_output();
    let Some(elem) = output.as_mut().get_mut(..slice.len()) else {
      return Err(CryptoError::InvalidHashLength.into());
    };
    elem.copy_from_slice(slice);
    Ok(GraviolaPrk { output, phantom: core::marker::PhantomData })
  }

  #[inline]
  fn compute<'data>(
    data: impl IntoIterator<Item = &'data [u8]>,
    key: &[u8],
  ) -> graviola::hashing::HashOutput {
    let mut hmac = graviola::hashing::hmac::Hmac::<H>::new(key);
    for chunk in data {
      hmac.update(chunk);
    }
    hmac.finish()
  }

  #[inline]
  fn expand(&self, info: &[u8], mut okm: &mut [u8]) -> crate::Result<()> {
    let len = okm.len();
    let hash_len = H::zeroed_output().as_ref().len();
    if len > hash_len.wrapping_mul(255) {
      return Err(CryptoError::LargeHkdfOutput.into());
    }
    #[expect(
      clippy::as_conversions,
      clippy::cast_possible_truncation,
      reason = "l <= 255 * hash_len <=> l / hash_len <= 255"
    )]
    let num = len.div_ceil(hash_len) as u8;
    let hmac_key = graviola::hashing::hmac::Hmac::<H>::new(&self.output);
    let mut hmac = hmac_key.clone();
    for idx in 1..=num {
      hmac.update(info);
      hmac.update([idx]);
      let hash = hmac.finish();
      let hash_slice = hash.as_ref();
      let min_len = okm.len().min(hash_slice.len());
      let (chunk, rest) = okm.split_at_mut(min_len);
      if let Some(elem) = hash_slice.get(..min_len) {
        chunk.copy_from_slice(elem);
      }
      okm = rest;
      if okm.is_empty() {
        return Ok(());
      }
      hmac = hmac_key.clone();
      hmac.update(hash_slice);
    }
    Ok(())
  }
}

/// Constructors shouldn't call this method because of scenarios where plaintext is used.
#[expect(clippy::panic, reason = "dummy structures should not be called")]
fn dummy_crypto_call() -> ! {
  panic!(
    "An operation required a crypto algorithm but no crypto backend was selected! You can, for example, enable the `crypto-ring` feature."
  );
}
