use crate::rng::{xor_u8, xor_u8_16, xor_u8_4, Rng};
use std::{
  collections::hash_map::RandomState,
  hash::{BuildHasher, Hasher},
};

/// Derived from the tools provided by the standard library, uses a simple XOR strategy.
#[derive(Debug)]
pub struct StdRng(u64);

impl Rng for StdRng {
  #[inline]
  fn u8(&mut self) -> u8 {
    xor_u8(&mut self.0)
  }

  #[inline]
  fn u8_4(&mut self) -> [u8; 4] {
    xor_u8_4(&mut self.0)
  }

  #[inline]
  fn u8_16(&mut self) -> [u8; 16] {
    xor_u8_16(&mut self.0)
  }
}

impl Default for StdRng {
  #[inline]
  fn default() -> Self {
    Self(Hasher::finish(&BuildHasher::build_hasher(&RandomState::new())))
  }
}
