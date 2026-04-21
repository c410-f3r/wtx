#![expect(deprecated, reason = "didn't find an alternative")]

use crate::{
  crypto::{CryptoError, Hmac, HmacSha256Ring, HmacSha384Ring},
  misc::unlikely_elem,
};
use ring::hmac::{Context, HMAC_SHA256, HMAC_SHA384, Key};

impl Hmac for HmacSha256Ring {
  type Digest = [u8; 32];

  #[inline]
  fn from_key(key: &[u8]) -> crate::Result<Self> {
    Ok(Self::new(Context::with_key(&Key::new(HMAC_SHA256, key))))
  }

  #[inline]
  fn update(&mut self, data: &[u8]) {
    self.0.update(data);
  }

  #[inline]
  fn digest(self) -> Self::Digest {
    let tag = self.0.sign();
    if let Ok(elem) = tag.as_ref().try_into() { elem } else { unlikely_elem([0; 32]) }
  }

  #[inline]
  fn verify(self, tag: &[u8]) -> crate::Result<()> {
    let computed = self.digest();
    if ring::constant_time::verify_slices_are_equal(&computed, tag).is_ok() {
      Ok(())
    } else {
      Err(CryptoError::HmacVerificationError.into())
    }
  }
}

impl Hmac for HmacSha384Ring {
  type Digest = [u8; 48];

  #[inline]
  fn from_key(key: &[u8]) -> crate::Result<Self> {
    Ok(Self::new(Context::with_key(&Key::new(HMAC_SHA384, key))))
  }

  #[inline]
  fn update(&mut self, data: &[u8]) {
    self.0.update(data);
  }

  #[inline]
  fn digest(self) -> Self::Digest {
    let tag = self.0.sign();
    if let Ok(elem) = tag.as_ref().try_into() { elem } else { unlikely_elem([0; 48]) }
  }

  #[inline]
  fn verify(self, tag: &[u8]) -> crate::Result<()> {
    let computed = self.digest();
    if ring::constant_time::verify_slices_are_equal(&computed, tag).is_ok() {
      Ok(())
    } else {
      Err(CryptoError::HmacVerificationError.into())
    }
  }
}
