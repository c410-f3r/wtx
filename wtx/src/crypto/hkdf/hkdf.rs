use crate::crypto::{CryptoError, Hkdf, HkdfSha256RustCrypto, HkdfSha384RustCrypto};
use crypto_common::{KeyInit, Output};
use hmac::{Mac, SimpleHmac};

impl Hkdf for HkdfSha256RustCrypto {
  type Digest = Output<sha2::Sha256>;

  #[inline]
  fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (Self::Digest, Self) {
    let (lhs, rhs) = hkdf::Hkdf::extract(salt, ikm);
    (lhs, Self(rhs))
  }

  #[inline]
  fn from_prk(prk: &[u8]) -> crate::Result<Self> {
    Ok(Self(hkdf::Hkdf::from_prk(prk).map_err(|_err| CryptoError::HkdfFromPrkError)?))
  }

  #[inline]
  fn compute<'data>(
    data: impl IntoIterator<Item = &'data [u8]>,
    key: &[u8],
  ) -> crate::Result<Self::Digest> {
    let mut hmac = SimpleHmac::<sha2::Sha256>::new_from_slice(key)?;
    for elem in data {
      Mac::update(&mut hmac, elem);
    }
    Ok(hmac.finalize().into_bytes())
  }

  #[inline]
  fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()> {
    hkdf::Hkdf::expand(&self.0, info, okm).map_err(|_err| CryptoError::HkdfExpandError)?;
    Ok(())
  }
}

impl Hkdf for HkdfSha384RustCrypto {
  type Digest = Output<sha2::Sha384>;

  #[inline]
  fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (Self::Digest, Self) {
    let (lhs, rhs) = hkdf::Hkdf::extract(salt, ikm);
    (lhs, Self(rhs))
  }

  #[inline]
  fn from_prk(prk: &[u8]) -> crate::Result<Self> {
    Ok(Self(hkdf::Hkdf::from_prk(prk).map_err(|_err| CryptoError::HkdfFromPrkError)?))
  }

  #[inline]
  fn compute<'data>(
    data: impl IntoIterator<Item = &'data [u8]>,
    key: &[u8],
  ) -> crate::Result<Self::Digest> {
    let mut hmac = SimpleHmac::<sha2::Sha384>::new_from_slice(key)?;
    for elem in data {
      Mac::update(&mut hmac, elem);
    }
    Ok(hmac.finalize().into_bytes())
  }

  #[inline]
  fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()> {
    hkdf::Hkdf::expand(&self.0, info, okm).map_err(|_err| CryptoError::HkdfExpandError)?;
    Ok(())
  }
}
