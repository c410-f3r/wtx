use crate::crypto::{Hmac, HmacSha256RustCrypto, HmacSha384RustCrypto};
use crypto_common::KeyInit;
use hmac::Mac;
use sha2::{Sha256, Sha384};

impl Hmac for HmacSha256RustCrypto {
  type Digest = [u8; 32];

  #[inline]
  fn from_key(key: &[u8]) -> crate::Result<Self> {
    Ok(Self::new(hmac::Hmac::<Sha256>::new_from_slice(key)?))
  }

  #[inline]
  fn update(&mut self, data: &[u8]) {
    self.0.update(data);
  }

  #[inline]
  fn digest(self) -> Self::Digest {
    self.0.finalize().into_bytes().into()
  }

  #[inline]
  fn verify(self, tag: &[u8]) -> crate::Result<()> {
    self.0.verify_slice(tag)?;
    Ok(())
  }
}

impl Hmac for HmacSha384RustCrypto {
  type Digest = [u8; 48];

  #[inline]
  fn from_key(key: &[u8]) -> crate::Result<Self> {
    Ok(Self::new(hmac::Hmac::<Sha384>::new_from_slice(key)?))
  }

  #[inline]
  fn update(&mut self, data: &[u8]) {
    self.0.update(data);
  }

  #[inline]
  fn digest(self) -> Self::Digest {
    self.0.finalize().into_bytes().into()
  }

  #[inline]
  fn verify(self, tag: &[u8]) -> crate::Result<()> {
    self.0.verify_slice(tag)?;
    Ok(())
  }
}
