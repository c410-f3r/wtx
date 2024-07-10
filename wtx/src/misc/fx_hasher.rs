use core::{hash::Hasher, ops::BitXor};

const K: u64 = 0x517c_c1b7_2722_0a95;

/// <https://nnethercote.github.io/2021/12/08/a-brutally-effective-hash-function-in-rust.html>
///
/// Has a fixed output standard, as such, it can be used in algorithms where a hash needs to be
/// sent over the network, or persisted.
#[derive(Debug)]
pub struct FxHasher(u64);

impl FxHasher {
  /// Creates a default instance.
  #[inline]
  pub const fn new() -> FxHasher {
    Self(0)
  }

  /// Creates a instance with a given seed.
  #[inline]
  pub const fn with_seed(seed: u64) -> FxHasher {
    Self(seed)
  }
}

impl Default for FxHasher {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl Hasher for FxHasher {
  #[inline]
  fn finish(&self) -> u64 {
    self.0
  }

  #[inline]
  fn write(&mut self, bytes: &[u8]) {
    for byte in bytes.iter().copied() {
      let into: u64 = byte.into();
      self.0 = self.0.rotate_left(5).bitxor(into).wrapping_mul(K);
    }
  }
}
