use crate::crypto::{CryptoError, Hkdf, HkdfRustCrypto};
use crypto_common::{KeyInit, Output};
use digest::OutputSizeUser;
use hmac::{EagerHash, Mac, SimpleHmac};

impl<H> Hkdf for HkdfRustCrypto<H>
where
  H: EagerHash,
  H::Core: OutputSizeUser<OutputSize = H::OutputSize>,
{
  type Digest = Output<H>;

  #[inline]
  fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (Self::Digest, Self) {
    let (lhs, rhs) = hkdf::Hkdf::<H>::extract(salt, ikm);
    (lhs, HkdfRustCrypto(rhs))
  }

  #[inline]
  fn from_prk(prk: &[u8]) -> crate::Result<Self> {
    Ok(HkdfRustCrypto(hkdf::Hkdf::from_prk(prk).map_err(|_err| CryptoError::HkdfFromPrkError)?))
  }

  #[inline]
  fn compute<'data>(
    data: impl IntoIterator<Item = &'data [u8]>,
    key: &[u8],
  ) -> crate::Result<Self::Digest> {
    let mut hmac = SimpleHmac::<H>::new_from_slice(key)?;
    for elem in data {
      Mac::update(&mut hmac, elem);
    }
    Ok(hmac.finalize().into_bytes())
  }

  #[inline]
  fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()> {
    hkdf::Hkdf::<H>::expand(&self.0, info, okm).map_err(|_err| CryptoError::HkdfExpandError)?;
    Ok(())
  }
}
