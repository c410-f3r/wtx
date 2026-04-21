use crate::{
  crypto::{Hmac, HmacSha256Graviola, HmacSha384Graviola},
  misc::unlikely_elem,
};
use graviola::hashing::{Sha256, Sha384, hmac};

impl Hmac for HmacSha256Graviola {
  type Digest = [u8; 32];

  #[inline]
  fn from_key(key: &[u8]) -> crate::Result<Self> {
    Ok(Self(hmac::Hmac::<Sha256>::new(key)))
  }

  #[inline]
  fn update(&mut self, data: &[u8]) {
    self.0.update(data);
  }

  #[inline]
  fn digest(self) -> Self::Digest {
    let tag = self.0.finish();
    if let Ok(elem) = tag.as_ref().try_into() { elem } else { unlikely_elem([0; 32]) }
  }

  #[inline]
  fn verify(self, tag: &[u8]) -> crate::Result<()> {
    self.0.verify(tag)?;
    Ok(())
  }
}

impl Hmac for HmacSha384Graviola {
  type Digest = [u8; 48];

  #[inline]
  fn from_key(key: &[u8]) -> crate::Result<Self> {
    Ok(Self(hmac::Hmac::<Sha384>::new(key)))
  }

  #[inline]
  fn update(&mut self, data: &[u8]) {
    self.0.update(data);
  }

  #[inline]
  fn digest(self) -> Self::Digest {
    let tag = self.0.finish();
    if let Ok(elem) = tag.as_ref().try_into() { elem } else { unlikely_elem([0; 48]) }
  }

  #[inline]
  fn verify(self, tag: &[u8]) -> crate::Result<()> {
    self.0.verify(tag)?;
    Ok(())
  }
}
