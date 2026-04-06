use crate::misc::unlikely_elem;
use graviola::hashing::{Hash as _, HashContext};

impl crate::crypto::Hash for crate::crypto::Sha256DigestGraviola {
  type Digest = [u8; 32];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    let mut ctx = graviola::hashing::Sha256::new();
    for elem in data {
      ctx.update(elem);
    }
    let rslt = ctx.finish();
    if let Ok(elem) = rslt.as_ref().try_into() { elem } else { unlikely_elem([0; 32]) }
  }
}

impl crate::crypto::Hash for crate::crypto::Sha384DigestGraviola {
  type Digest = [u8; 48];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    let mut ctx = graviola::hashing::Sha384::new();
    for elem in data {
      ctx.update(elem);
    }
    let rslt = ctx.finish();
    if let Ok(elem) = rslt.as_ref().try_into() { elem } else { unlikely_elem([0; 48]) }
  }
}
