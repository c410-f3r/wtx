//! Algorithms that prevent third parties or the public from reading private messages.
//!
//! The structures available in this module are intended for internal operations but they can be
//! useful for public utilization.

#[macro_use]
mod macros;

mod aead;
mod agreement;
mod crypto_error;
mod dyn_signing_key;
mod dyn_signing_output;
mod hash;
mod hkdf;
mod hmac;
mod signing_key;
mod signing_output;

use crate::rng::CryptoRng;
pub use aead::{
  Aead, AeadDummy,
  global::{Aes128GcmGlobal, Aes256GcmGlobal, Chacha20Poly1305Global},
};
pub use agreement::{
  Agreement, AgreementDummy,
  global::{EcdhP256Global, EcdhP384Global, X25519Global},
};
pub use crypto_error::CryptoError;
pub use dyn_signing_key::DynSigningKey;
pub use dyn_signing_output::DynSigningOutput;
pub use hash::{
  Hash, HashDummy,
  global::{Sha1Global, Sha256Global, Sha384Global, Sha512Global},
  hash_ty::HashTy,
};
pub use hkdf::{
  Hkdf, HkdfDummy,
  global::{HkdfSha256Global, HkdfSha384Global},
};
pub use hmac::{
  Hmac, HmacDummy,
  global::{HmacSha256Global, HmacSha384Global},
};
pub use signing_key::{
  SigningKey, SigningKeyDummy,
  global::{
    EcdsaP256SigningKeyGlobal, EcdsaP384SigningKeyGlobal, Ed25519SigningKeyGlobal,
    RsaPkcs1SigningKeyGlobal, RsaPssSigningKeyGlobal,
  },
};
pub use signing_output::SigningOutput;

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

/// A wrapper around external structures that don't implement `AsRef<[u8]>`.
#[cfg(any(feature = "crypto-graviola", feature = "crypto-ruco"))]
#[derive(Debug)]
pub struct AsRefWrapper<T>(T);

#[cfg(feature = "crypto-alr")]
_create_wrappers!(
  /// Aead
  #[derive(Default)]
  Aes128GcmAlr<>(),
  #[derive(Default)]
  Aes256GcmAlr<>(),
  #[derive(Default)]
  Chacha20Poly1305Alr<>(),
  // Agreement
  EcdhP256Alr<>(aws_lc_rs::agreement::EphemeralPrivateKey),
  EcdhP384Alr<>(aws_lc_rs::agreement::EphemeralPrivateKey),
  X25519Alr<>(aws_lc_rs::agreement::EphemeralPrivateKey),
  // Hash
  #[derive(Clone)]
  Sha1Alr<>(aws_lc_rs::digest::Context),
  #[derive(Clone)]
  Sha256Alr<>(aws_lc_rs::digest::Context),
  #[derive(Clone)]
  Sha384Alr<>(aws_lc_rs::digest::Context),
  #[derive(Clone)]
  Sha512Alr<>(aws_lc_rs::digest::Context),
  // Hkdf
  HkdfSha256Alr<>(aws_lc_rs::hkdf::Prk),
  HkdfSha384Alr<>(aws_lc_rs::hkdf::Prk),
  // Hmac
  HmacSha256Alr<>(aws_lc_rs::hmac::Context),
  HmacSha384Alr<>(aws_lc_rs::hmac::Context),
  // Signature
  #[derive(Default)]
  EcdsaP256Alr<>(),
  #[derive(Default)]
  EcdsaP384Alr<>(),
  #[derive(Default)]
  Ed25519Alr<>(),
  #[derive(Default)]
  RsaPssAlr<>(),
  // Signing Key
  EcdsaP256SigningKeyAlr<>(aws_lc_rs::signature::EcdsaKeyPair),
  EcdsaP384SigningKeyAlr<>(aws_lc_rs::signature::EcdsaKeyPair),
  Ed25519SigningKeyAlr<>(aws_lc_rs::signature::Ed25519KeyPair),
  RsaPkcs1SigningKeyAlr<>((HashTy, aws_lc_rs::signature::RsaKeyPair)),
  RsaPssSigningKeyAlr<>((HashTy, aws_lc_rs::signature::RsaKeyPair)),
);

#[cfg(feature = "crypto-graviola")]
_create_wrappers!(
  // Aead
  #[derive(Default)]
  Aes128GcmGraviola<>(),
  #[derive(Default)]
  Aes256GcmGraviola<>(),
  #[derive(Default)]
  Chacha20Poly1305Graviola<>(),
  // Agreement
  EcdhP256Graviola<>(graviola::key_agreement::p256::PrivateKey),
  EcdhP384Graviola<>(graviola::key_agreement::p384::PrivateKey),
  X25519Graviola<>(graviola::key_agreement::x25519::PrivateKey),
  // Hash
  #[derive(Clone)]
  Sha256Graviola<>(<graviola::hashing::Sha256 as graviola::hashing::Hash>::Context),
  #[derive(Clone)]
  Sha384Graviola<>(<graviola::hashing::Sha384 as graviola::hashing::Hash>::Context),
  #[derive(Clone)]
  Sha512Graviola<>(<graviola::hashing::Sha512 as graviola::hashing::Hash>::Context),
  // Hkdf
  HkdfSha256Graviola<>(GraviolaPrk<graviola::hashing::Sha256>),
  HkdfSha384Graviola<>(GraviolaPrk<graviola::hashing::Sha384>),
  // Hmac
  HmacSha256Graviola<>(graviola::hashing::hmac::Hmac<graviola::hashing::Sha256>),
  HmacSha384Graviola<>(graviola::hashing::hmac::Hmac<graviola::hashing::Sha384>),
  // Signature
  #[derive(Default)]
  EcdsaP256Graviola<>(),
  #[derive(Default)]
  EcdsaP384Graviola<>(),
  #[derive(Default)]
  Ed25519Graviola<>(),
  #[derive(Default)]
  RsaPssGraviola<>(),
  // Signing Key
  EcdsaP256SigningKeyGraviola<>(graviola::signing::ecdsa::SigningKey<graviola::signing::ecdsa::P256>),
  EcdsaP384SigningKeyGraviola<>(graviola::signing::ecdsa::SigningKey<graviola::signing::ecdsa::P384>),
  Ed25519SigningKeyGraviola<>(graviola::signing::eddsa::Ed25519SigningKey),
  RsaPkcs1SigningKeyGraviola<>((HashTy, graviola::signing::rsa::SigningKey)),
  RsaPssSigningKeyGraviola<>((HashTy, graviola::signing::rsa::SigningKey)),
);

#[cfg(feature = "crypto-ring")]
_create_wrappers!(
  // Aead
  #[derive(Default)]
  Aes128GcmRing<>(),
  #[derive(Default)]
  Aes256GcmRing<>(),
  #[derive(Default)]
  Chacha20Poly1305Ring<>(),
  // Agreement
  EcdhP256Ring<>(ring::agreement::EphemeralPrivateKey),
  EcdhP384Ring<>(ring::agreement::EphemeralPrivateKey),
  X25519Ring<>(ring::agreement::EphemeralPrivateKey),
  // Hash
  #[derive(Clone)]
  Sha1Ring<>(ring::digest::Context),
  #[derive(Clone)]
  Sha256Ring<>(ring::digest::Context),
  #[derive(Clone)]
  Sha384Ring<>(ring::digest::Context),
  #[derive(Clone)]
  Sha512Ring<>(ring::digest::Context),
  // Hkdf
  HkdfSha256Ring<>(ring::hkdf::Prk),
  HkdfSha384Ring<>(ring::hkdf::Prk),
  // Hmac
  HmacSha256Ring<>(ring::hmac::Context),
  HmacSha384Ring<>(ring::hmac::Context),
  // Signature
  #[derive(Default)]
  EcdsaP256Ring<>(),
  #[derive(Default)]
  EcdsaP384Ring<>(),
  #[derive(Default)]
  Ed25519Ring<>(),
  #[derive(Default)]
  RsaPssRing<>(),
  // Signing Key
  EcdsaP256SigningKeyRing<>(ring::signature::EcdsaKeyPair),
  EcdsaP384SigningKeyRing<>(ring::signature::EcdsaKeyPair),
  Ed25519SigningKeyRing<>(ring::signature::Ed25519KeyPair),
  RsaPkcs1SigningKeyRing<>((HashTy, ring::signature::RsaKeyPair)),
  RsaPssSigningKeyRing<>((HashTy, ring::signature::RsaKeyPair)),
);

#[cfg(feature = "crypto-ruco")]
_create_wrappers!(
  // Aead
  #[derive(Default)]
  Aes128GcmRuco<>(),
  #[derive(Default)]
  Aes256GcmRuco<>(),
  #[derive(Default)]
  Chacha20Poly1305Ruco<>(),
  // Agreement
  EcdhP256Ruco<>(p256::ecdh::EphemeralSecret),
  EcdhP384Ruco<>(p384::ecdh::EphemeralSecret),
  X25519Ruco<>(x25519_dalek::EphemeralSecret),
  // Hash
  #[derive(Clone)]
  Sha1Ruco<>(sha1::Sha1),
  #[derive(Clone)]
  Sha256Ruco<>(sha2::Sha256),
  #[derive(Clone)]
  Sha384Ruco<>(sha2::Sha384),
  #[derive(Clone)]
  Sha512Ruco<>(sha2::Sha512),
  // Hkdf
  HkdfSha256Ruco<>(::hkdf::Hkdf<sha2::Sha256>),
  HkdfSha384Ruco<>(::hkdf::Hkdf<sha2::Sha384>),
  // Hmac
  HmacSha256Ruco<>(::hmac::Hmac<sha2::Sha256>),
  HmacSha384Ruco<>(::hmac::Hmac<sha2::Sha384>),
  // Signature
  #[derive(Default)]
  EcdsaP256Ruco<>(),
  #[derive(Default)]
  EcdsaP384Ruco<>(),
  #[derive(Default)]
  Ed25519Ruco<>(),
  #[derive(Default)]
  RsaPssRuco<>(),
  // Signing Key
  EcdsaP256SigningKeyRuco<>(p256::ecdsa::SigningKey),
  EcdsaP384SigningKeyRuco<>(p384::ecdsa::SigningKey),
  Ed25519SigningKeyRuco<>(ed25519_dalek::SigningKey),
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
    let num = len.div_ceil(hash_len).try_into().unwrap_or_default();
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

/// Dynamaic RSA PKCS1 Signing key from the Rust Crypto
#[cfg(feature = "crypto-ruco")]
#[derive(Debug)]
pub enum RsaPkcs1SigningKeyRuco {
  /// See [`rsa::pkcs1v15::SigningKey`]
  Sha256(rsa::pkcs1v15::SigningKey<sha2::Sha256>),
  /// See [`rsa::pkcs1v15::SigningKey`]
  Sha384(rsa::pkcs1v15::SigningKey<sha2::Sha384>),
  /// See [`rsa::pkcs1v15::SigningKey`]
  Sha512(rsa::pkcs1v15::SigningKey<sha2::Sha512>),
}

/// Dynamaic RSA PSS Signing key from the Rust Crypto
#[cfg(feature = "crypto-ruco")]
#[derive(Debug)]
pub enum RsaPssSigningKeyRuco {
  /// See [`rsa::pss::SigningKey`]
  Sha256(rsa::pss::SigningKey<sha2::Sha256>),
  /// See [`rsa::pss::SigningKey`]
  Sha384(rsa::pss::SigningKey<sha2::Sha384>),
  /// See [`rsa::pss::SigningKey`]
  Sha512(rsa::pss::SigningKey<sha2::Sha512>),
}

/// Constructors shouldn't call this method because of scenarios where plaintext is used.
#[expect(clippy::panic, reason = "dummy structures should not be called")]
fn dummy_crypto_call() -> ! {
  panic!(
    "An operation required a crypto algorithm but no crypto backend was selected! You can, for example, enable the `crypto-ring` feature."
  );
}
