use crate::misc::Rng;
use core::sync::atomic::{AtomicU64, Ordering};

/// Xorshift that deals with 64 bits numbers.
#[derive(Clone, Copy, Debug)]
pub struct Xorshift64 {
  value: u64,
}

#[cfg(feature = "http-server-framework")]
impl crate::http::server_framework::ConnAux for Xorshift64 {
  type Init = Self;

  #[inline]
  fn conn_aux(init: Self::Init) -> crate::Result<Self> {
    Ok(init)
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

impl From<u64> for Xorshift64 {
  #[inline]
  fn from(value: u64) -> Self {
    Self { value }
  }
}

/// Xorshift that deals with 64 bits numbers.
///
/// Suitable for multi-thread environments.
#[derive(Debug)]
pub struct Xorshift64Sync {
  value: AtomicU64,
}

impl Xorshift64Sync {
  #[inline]
  fn modify(&self) -> u64 {
    self
      .value
      .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |mut el| {
        let _ = xor_numbers(&mut el);
        Some(el)
      })
      .unwrap_or_else(|el| el)
  }
}

impl Rng for Xorshift64Sync {
  #[inline]
  fn u8(&mut self) -> u8 {
    u8(self.modify())
  }

  #[inline]
  fn u8_4(&mut self) -> [u8; 4] {
    u8_4(self.modify())
  }

  #[inline]
  fn u8_8(&mut self) -> [u8; 8] {
    u8_8(self.modify())
  }

  #[inline]
  fn u8_16(&mut self) -> [u8; 16] {
    u8_16(self.modify(), self.modify())
  }
}

impl Rng for &Xorshift64Sync {
  #[inline]
  fn u8(&mut self) -> u8 {
    u8(self.modify())
  }

  #[inline]
  fn u8_4(&mut self) -> [u8; 4] {
    u8_4(self.modify())
  }

  #[inline]
  fn u8_8(&mut self) -> [u8; 8] {
    u8_8(self.modify())
  }

  #[inline]
  fn u8_16(&mut self) -> [u8; 16] {
    u8_16(self.modify(), self.modify())
  }
}

impl From<u64> for Xorshift64Sync {
  #[inline]
  fn from(value: u64) -> Self {
    Self { value: AtomicU64::new(value) }
  }
}

#[inline]
fn u8(n: u64) -> u8 {
  let [a, ..] = n.to_be_bytes();
  a
}

#[inline]
fn u8_4(n: u64) -> [u8; 4] {
  let [a, b, c, d, ..] = n.to_be_bytes();
  [a, b, c, d]
}

#[inline]
fn u8_8(n: u64) -> [u8; 8] {
  n.to_be_bytes()
}

#[inline]
fn u8_16(first: u64, second: u64) -> [u8; 16] {
  let [a, b, c, d, e, f, g, h] = first.to_be_bytes();
  let [i, j, k, l, m, n, o, p] = second.to_be_bytes();
  [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p]
}

#[inline]
fn xor_numbers(seed: &mut u64) -> u64 {
  *seed ^= *seed << 13;
  *seed ^= *seed >> 17;
  *seed ^= *seed << 5;
  *seed
}

#[inline]
fn xor_u8(seed: &mut u64) -> u8 {
  u8(xor_numbers(seed))
}

#[inline]
fn xor_u8_4(seed: &mut u64) -> [u8; 4] {
  u8_4(xor_numbers(seed))
}

#[inline]
fn xor_u8_8(seed: &mut u64) -> [u8; 8] {
  u8_8(xor_numbers(seed))
}

#[inline]
fn xor_u8_16(seed: &mut u64) -> [u8; 16] {
  u8_16(xor_numbers(seed), xor_numbers(seed))
}
