use crate::{crypto::Hash, misc::unlikely_elem};
use ring::digest::Context;

impl Hash for crate::crypto::Sha1DigestRing {
  type Digest = [u8; 20];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    let mut context = Context::new(&ring::digest::SHA1_FOR_LEGACY_USE_ONLY);
    for elem in data {
      context.update(elem);
    }
    let rlst = context.finish();
    if let Ok(elem) = rlst.as_ref().try_into() { elem } else { unlikely_elem([0; 20]) }
  }
}

impl Hash for crate::crypto::Sha256DigestRing {
  type Digest = [u8; 32];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    let mut context = Context::new(&ring::digest::SHA256);
    for elem in data {
      context.update(elem);
    }
    let rlst = context.finish();
    if let Ok(elem) = rlst.as_ref().try_into() { elem } else { unlikely_elem([0; 32]) }
  }
}

impl Hash for crate::crypto::Sha384DigestRing {
  type Digest = [u8; 48];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    let mut context = Context::new(&ring::digest::SHA384);
    for elem in data {
      context.update(elem);
    }
    let rlst = context.finish();
    if let Ok(elem) = rlst.as_ref().try_into() { elem } else { unlikely_elem([0; 48]) }
  }
}
