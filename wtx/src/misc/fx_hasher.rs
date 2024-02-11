use core::{hash::Hasher, ops::BitXor};

const K: u64 = 0x517c_c1b7_2722_0a95;

/// https://nnethercote.github.io/2021/12/08/a-brutally-effective-hash-function-in-rust.html
///
/// Has a fixed output standard, as such, it can be used in algorithms where a hash needs to be
/// sent over the network, or persisted.
#[derive(Debug, Default)]
pub struct FxHasher(u64);

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
