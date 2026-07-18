use crate::misc::unlikely_elem;
use graviola::hashing::{Hash as _, HashContext};

impl crate::crypto::Hash for crate::crypto::HashSha256Graviola {
  type Digest = [u8; 32];

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

impl crate::crypto::Hash for crate::crypto::HashSha384Graviola {
  type Digest = [u8; 48];

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
