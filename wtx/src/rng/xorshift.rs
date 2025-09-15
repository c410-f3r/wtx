use crate::{
  misc::{Lease, LeaseMut},
  rng::{Rng, SeedableRng},
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
  pub const fn new(value: u64) -> Self {
    Self { value }
  }
}

impl Rng for Xorshift64 {
  #[inline]
  fn u8(&mut self) -> u8 {
    xor_u8(&mut self.value)
  }

  #[inline]
  fn u8_4(&mut self) -> [u8; 4] {
    xor_u8_4(&mut self.value)
  }

  #[inline]
  fn u8_8(&mut self) -> [u8; 8] {
    xor_u8_8(&mut self.value)
  }

  #[inline]
  fn u8_16(&mut self) -> [u8; 16] {
    xor_u8_16(&mut self.value)
  }
}

impl SeedableRng for Xorshift64 {
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

impl From<u64> for Xorshift64 {
  #[inline]
  fn from(value: u64) -> Self {
    Self { value }
  }
}

fn u8(n: u64) -> u8 {
  let [a, ..] = n.to_be_bytes();
  a
}

fn u8_4(n: u64) -> [u8; 4] {
  let [a, b, c, d, ..] = n.to_be_bytes();
  [a, b, c, d]
}

fn u8_8(n: u64) -> [u8; 8] {
  n.to_be_bytes()
}

fn u8_16(first: u64, second: u64) -> [u8; 16] {
  let [a, b, c, d, e, f, g, h] = first.to_be_bytes();
  let [i, j, k, l, m, n, o, p] = second.to_be_bytes();
  [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p]
}

fn xor_numbers(seed: &mut u64) -> u64 {
  *seed ^= *seed << 13;
  *seed ^= *seed >> 17;
  *seed ^= *seed << 5;
  *seed
}

fn xor_u8(seed: &mut u64) -> u8 {
  u8(xor_numbers(seed))
}

fn xor_u8_4(seed: &mut u64) -> [u8; 4] {
  u8_4(xor_numbers(seed))
}

fn xor_u8_8(seed: &mut u64) -> [u8; 8] {
  u8_8(xor_numbers(seed))
}

fn xor_u8_16(seed: &mut u64) -> [u8; 16] {
  u8_16(xor_numbers(seed), xor_numbers(seed))
}

#[cfg(feature = "http-server-framework")]
mod http_server_framework {
  use crate::{http::server_framework::ConnAux, rng::Xorshift64};

  impl ConnAux for Xorshift64 {
    type Init = Self;

    fn conn_aux(init: Self::Init) -> crate::Result<Self> {
      Ok(init)
    }
  }
}
