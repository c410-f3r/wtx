use crate::{
  misc::{Lease, LeaseMut},
  rng::{CryptoSeedableRng, Rng, SeedableRng},
};

/// Xorshift that deals with 64 bits numbers.
///
/// This structure is `Copy` to allow usage with `AtomicCell` in concurrent scenarios.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Xorshift64 {
  value: u64,
}

impl Xorshift64 {
  /// Constructor
  #[inline]
  pub const fn new(value: u64) -> Self {
    Self { value }
  }

  /// Next iteration value
  #[inline]
  pub const fn next_value(&mut self) -> u64 {
    self.value ^= self.value << 13;
    self.value ^= self.value >> 17;
    self.value ^= self.value << 5;
    self.value
  }
}

impl Rng for Xorshift64 {
  #[inline]
  fn u8_4(&mut self) -> [u8; 4] {
    u8_4(self.next_value())
  }

  #[inline]
  fn u8_8(&mut self) -> [u8; 8] {
    u8_8(self.next_value())
  }

  #[inline]
  fn u8_16(&mut self) -> [u8; 16] {
    u8_16(self.next_value(), self.next_value())
  }

  #[inline]
  fn u8_32(&mut self) -> [u8; 32] {
    u8_32(self.next_value(), self.next_value(), self.next_value(), self.next_value())
  }
}

impl CryptoSeedableRng for Xorshift64 {
  type Seed = [u8; 8];

  #[inline]
  fn from_seed(seed: Self::Seed) -> crate::Result<Self> {
    Ok(Self::from(u64::from_be_bytes(seed)))
  }
}

impl Lease<Xorshift64> for Xorshift64 {
  #[inline]
  fn lease(&self) -> &Xorshift64 {
    self
  }
}

impl LeaseMut<Xorshift64> for Xorshift64 {
  #[inline]
  fn lease_mut(&mut self) -> &mut Xorshift64 {
    self
  }
}

impl SeedableRng for Xorshift64 {}

impl From<u64> for Xorshift64 {
  #[inline]
  fn from(value: u64) -> Self {
    Self { value }
  }
}

const fn u8_4(n: u64) -> [u8; 4] {
  let [b0, b1, b2, b3, ..] = n.to_be_bytes();
  [b0, b1, b2, b3]
}

const fn u8_8(n: u64) -> [u8; 8] {
  n.to_be_bytes()
}

const fn u8_16(first: u64, second: u64) -> [u8; 16] {
  let [b0, b1, b2, b3, b4, b5, b6, b7] = first.to_be_bytes();
  let [b8, b9, b10, b11, b12, b13, b14, b15] = second.to_be_bytes();
  [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15]
}

const fn u8_32(first: u64, second: u64, third: u64, forth: u64) -> [u8; 32] {
  let [b0, b1, b2, b3, b4, b5, b6, b7] = first.to_be_bytes();
  let [b8, b9, b10, b11, b12, b13, b14, b15] = second.to_be_bytes();
  let [b16, b17, b18, b19, b20, b21, b22, b23] = third.to_be_bytes();
  let [b24, b25, b26, b27, b28, b29, b30, b31] = forth.to_be_bytes();
  [
    b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16, b17, b18, b19, b20,
    b21, b22, b23, b24, b25, b26, b27, b28, b29, b30, b31,
  ]
}
