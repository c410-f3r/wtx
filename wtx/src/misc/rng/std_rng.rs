use crate::misc::rng::{xor_numbers, xor_u8, xor_u8_16, xor_u8_4, xor_u8_8, Rng};
use core::{
  hash::{BuildHasher, Hasher},
  sync::atomic::{AtomicU64, Ordering},
};
use std::collections::hash_map::RandomState;

/// Derived from the tools provided by the standard library, uses a simple XOR strategy.
#[derive(Clone, Copy, Debug)]
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
  fn u8_8(&mut self) -> [u8; 8] {
    xor_u8_8(&mut self.0)
  }

  #[inline]
  fn u8_16(&mut self) -> [u8; 16] {
    xor_u8_16(&mut self.0)
  }
}

#[cfg(feature = "http-server-framework")]
impl crate::http::server_framework::ConnAux for StdRng {
  type Init = Self;

  #[inline]
  fn conn_aux(init: Self::Init) -> crate::Result<Self> {
    Ok(init)
  }
}

impl Default for StdRng {
  #[inline]
  fn default() -> Self {
    Self(Hasher::finish(&BuildHasher::build_hasher(&RandomState::new())))
  }
}

/// Synchronous version of [`StdRng`].
#[derive(Debug)]
pub struct StdRngSync(AtomicU64);

impl StdRngSync {
  #[inline]
  fn modify(&self) -> u64 {
    self
      .0
      .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |mut el| {
        let _ = xor_numbers(&mut el);
        Some(el)
      })
      .unwrap_or_else(|el| el)
  }
}

impl Rng for StdRngSync {
  #[inline]
  fn u8(&mut self) -> u8 {
    crate::misc::rng::u8(self.modify())
  }

  #[inline]
  fn u8_4(&mut self) -> [u8; 4] {
    crate::misc::rng::u8_4(self.modify())
  }

  #[inline]
  fn u8_8(&mut self) -> [u8; 8] {
    crate::misc::rng::u8_8(self.modify())
  }

  #[inline]
  fn u8_16(&mut self) -> [u8; 16] {
    crate::misc::rng::u8_16(self.modify(), self.modify())
  }
}

impl Rng for &StdRngSync {
  #[inline]
  fn u8(&mut self) -> u8 {
    crate::misc::rng::u8(self.modify())
  }

  #[inline]
  fn u8_4(&mut self) -> [u8; 4] {
    crate::misc::rng::u8_4(self.modify())
  }

  #[inline]
  fn u8_8(&mut self) -> [u8; 8] {
    crate::misc::rng::u8_8(self.modify())
  }

  #[inline]
  fn u8_16(&mut self) -> [u8; 16] {
    crate::misc::rng::u8_16(self.modify(), self.modify())
  }
}

impl Default for StdRngSync {
  #[inline]
  fn default() -> Self {
    Self(AtomicU64::new(Hasher::finish(&BuildHasher::build_hasher(&RandomState::new()))))
  }
}
