use crate::crypto::{Hash, HashSha1Ruco, HashSha256Ruco, HashSha384Ruco};
use digest::Digest;

impl Hash for HashSha1Ruco {
  type Digest = [u8; 20];

  #[inline]
  fn new() -> Self {
    Self(<sha1::Sha1 as Digest>::new())
  }

  #[inline]
  fn finalize(self) -> Self::Digest {
    self.0.finalize().into()
  }

  #[inline]
  fn update(&mut self, data: &[u8]) {
    self.0.update(data);
  }
}

impl Hash for HashSha256Ruco {
  type Digest = [u8; 32];

  #[inline]
  fn new() -> Self {
    Self(<sha2::Sha256 as Digest>::new())
  }

  #[inline]
  fn finalize(self) -> Self::Digest {
    self.0.finalize().into()
  }

  #[inline]
  fn update(&mut self, data: &[u8]) {
    self.0.update(data);
  }
}

impl Hash for HashSha384Ruco {
  type Digest = [u8; 48];

  #[inline]
  fn new() -> Self {
    Self(<sha2::Sha384 as Digest>::new())
  }

  #[inline]
  fn finalize(self) -> Self::Digest {
    self.0.finalize().into()
  }

  #[inline]
  fn update(&mut self, data: &[u8]) {
    self.0.update(data);
  }
}
