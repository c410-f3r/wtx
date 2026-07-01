use crate::{crypto::Hash, misc::unlikely_elem};
use ring::digest::{Context, SHA1_FOR_LEGACY_USE_ONLY, SHA256, SHA384};

impl Hash for crate::crypto::Sha1HashRing {
  type Digest = [u8; 20];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    let mut context = Context::new(&SHA1_FOR_LEGACY_USE_ONLY);
    for elem in data {
      context.update(elem);
    }
    finish_context(context, [0; 20])
  }

  #[inline]
  fn new() -> Self {
    Self(Context::new(&SHA1_FOR_LEGACY_USE_ONLY))
  }

  #[inline]
  fn finalize(self) -> Self::Digest {
    finish_context(self.0, [0; 20])
  }

  #[inline]
  fn update(&mut self, data: &[u8]) {
    self.0.update(data);
  }
}

impl Hash for crate::crypto::Sha256HashRing {
  type Digest = [u8; 32];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    let mut context = Context::new(&SHA256);
    for elem in data {
      context.update(elem);
    }
    finish_context(context, [0; 32])
  }

  #[inline]
  fn new() -> Self {
    Self(Context::new(&SHA256))
  }

  #[inline]
  fn finalize(self) -> Self::Digest {
    finish_context(self.0, [0; 32])
  }

  #[inline]
  fn update(&mut self, data: &[u8]) {
    self.0.update(data);
  }
}

impl Hash for crate::crypto::Sha384HashRing {
  type Digest = [u8; 48];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    let mut context = Context::new(&SHA384);
    for elem in data {
      context.update(elem);
    }
    finish_context(context, [0; 48])
  }

  #[inline]
  fn new() -> Self {
    Self(Context::new(&SHA384))
  }

  #[inline]
  fn finalize(self) -> Self::Digest {
    finish_context(self.0, [0; 48])
  }

  #[inline]
  fn update(&mut self, data: &[u8]) {
    self.0.update(data);
  }
}

#[inline]
fn finish_context<const N: usize>(context: Context, default: [u8; N]) -> [u8; N] {
  let rslt = context.finish();
  if let Ok(elem) = rslt.as_ref().try_into() { elem } else { unlikely_elem(default) }
}
