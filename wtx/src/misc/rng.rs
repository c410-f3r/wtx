//! Random Number Generators

#[cfg(feature = "fastrand")]
mod fastrand;
mod no_std_rng;
#[cfg(feature = "rand")]
mod rand;
#[cfg(feature = "std")]
mod std_rng;

#[cfg(feature = "std")]
pub use self::std_rng::{StdRng, StdRngSync};
use crate::misc::Usize;
use core::{
  cell::Cell,
  ops::{Bound, RangeBounds},
};
pub use no_std_rng::NoStdRng;

/// Allows the creation of random instances.
pub trait FromRng<RNG>
where
  RNG: Rng,
{
  /// Creates a new instance based on `rng`.
  fn from_rng(rng: &mut RNG) -> Self;
}

impl<RNG> FromRng<RNG> for u8
where
  RNG: Rng,
{
  #[inline]
  fn from_rng(rng: &mut RNG) -> Self {
    rng.u8_8()[0]
  }
}

impl<RNG> FromRng<RNG> for usize
where
  RNG: Rng,
{
  #[inline]
  fn from_rng(rng: &mut RNG) -> Self {
    Usize::from_u64(u64::from_be_bytes(rng.u8_8()))
      .unwrap_or_else(|| Usize::from_u32(u32::from_be_bytes(rng.u8_4())))
      .into_usize()
  }
}

/// Abstraction tailored for the needs of this project. Each implementation should manage how
/// seeds are retrieved as well as how numbers are generated.
pub trait Rng
where
  Self: Sized,
{
  /// Creates an element that is within the given `range`.
  #[inline]
  fn elem_from_range<R, T>(&mut self, range: &R) -> Option<T>
  where
    R: RangeBounds<T>,
    T: FromRng<Self> + PartialOrd,
  {
    match (range.start_bound(), range.end_bound()) {
      (Bound::Included(a) | Bound::Excluded(a), Bound::Included(b)) => {
        if a < b {
          return None;
        }
      }
      (Bound::Included(a) | Bound::Excluded(a), Bound::Excluded(b)) => {
        if a <= b {
          return None;
        }
      }
      _ => {}
    }
    loop {
      let random = T::from_rng(self);
      if range.contains(&random) {
        return Some(random);
      }
    }
  }

  /// Fills `slice` with random data.
  #[inline]
  fn fill_slice<T>(&mut self, slice: &mut [T])
  where
    T: FromRng<Self>,
  {
    for elem in slice {
      *elem = T::from_rng(self);
    }
  }

  /// Shuffles a mutable slice in place.
  #[inline]
  fn shuffle_slice<T>(&mut self, slice: &mut [T]) {
    if slice.len() <= 1 {
      return;
    }
    for from_idx in 0..slice.len() {
      let range = 0..from_idx.wrapping_add(1);
      let Some(to_idx) = self.elem_from_range(&range) else {
        continue;
      };
      slice.swap(from_idx, to_idx);
    }
  }

  /// Creates an byte
  fn u8(&mut self) -> u8;

  /// Creates an array of 4 bytes.
  fn u8_4(&mut self) -> [u8; 4];

  /// Creates an array of 8 bytes.
  fn u8_8(&mut self) -> [u8; 8];

  /// Creates an array of 16 bytes.
  fn u8_16(&mut self) -> [u8; 16];
}

impl<T> Rng for Cell<T>
where
  T: Copy + Rng,
{
  #[inline]
  fn u8(&mut self) -> u8 {
    let mut instance = self.get();
    let rslt = instance.u8();
    self.set(instance);
    rslt
  }

  #[inline]
  fn u8_4(&mut self) -> [u8; 4] {
    let mut instance = self.get();
    let rslt = instance.u8_4();
    self.set(instance);
    rslt
  }

  #[inline]
  fn u8_8(&mut self) -> [u8; 8] {
    let mut instance = self.get();
    let rslt = instance.u8_8();
    self.set(instance);
    rslt
  }

  #[inline]
  fn u8_16(&mut self) -> [u8; 16] {
    let mut instance = self.get();
    let rslt = instance.u8_16();
    self.set(instance);
    rslt
  }
}

impl<T> Rng for &Cell<T>
where
  T: Copy + Rng,
{
  #[inline]
  fn u8(&mut self) -> u8 {
    self.get().u8()
  }

  #[inline]
  fn u8_4(&mut self) -> [u8; 4] {
    self.get().u8_4()
  }

  #[inline]
  fn u8_8(&mut self) -> [u8; 8] {
    self.get().u8_8()
  }

  #[inline]
  fn u8_16(&mut self) -> [u8; 16] {
    self.get().u8_16()
  }
}

impl<T> Rng for &mut T
where
  T: Rng,
{
  #[inline]
  fn u8(&mut self) -> u8 {
    (*self).u8()
  }

  #[inline]
  fn u8_4(&mut self) -> [u8; 4] {
    (*self).u8_4()
  }

  #[inline]
  fn u8_8(&mut self) -> [u8; 8] {
    (*self).u8_8()
  }

  #[inline]
  fn u8_16(&mut self) -> [u8; 16] {
    (*self).u8_16()
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
