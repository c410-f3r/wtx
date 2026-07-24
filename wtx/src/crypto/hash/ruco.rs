use crate::crypto::{Hash, Sha1Ruco, Sha256Ruco, Sha384Ruco};
use digest::Digest;

impl Hash for Sha1Ruco {
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

impl Hash for Sha256Ruco {
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

impl Hash for Sha384Ruco {
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
