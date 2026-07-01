use crate::misc::unlikely_elem;
use graviola::hashing::{Hash as _, HashContext};

impl crate::crypto::Hash for crate::crypto::Sha256HashGraviola {
  type Digest = [u8; 32];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    let mut ctx = graviola::hashing::Sha256::new();
    for elem in data {
      ctx.update(elem);
    }
    finish_context(ctx, [0; 32])
  }

  #[inline]
  fn new() -> Self {
    Self(graviola::hashing::Sha256::new())
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

impl crate::crypto::Hash for crate::crypto::Sha384HashGraviola {
  type Digest = [u8; 48];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    let mut ctx = graviola::hashing::Sha384::new();
    for elem in data {
      ctx.update(elem);
    }
    finish_context(ctx, [0; 48])
  }

  #[inline]
  fn new() -> Self {
    Self(graviola::hashing::Sha384::new())
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
fn finish_context<C, const N: usize>(ctx: C, default: [u8; N]) -> [u8; N]
where
  C: HashContext,
{
  let rslt = ctx.finish();
  if let Ok(elem) = rslt.as_ref().try_into() { elem } else { unlikely_elem(default) }
}
