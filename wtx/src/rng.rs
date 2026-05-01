//! Random Number Generators

mod cha_cha20;
mod crypto_rng;
mod crypto_seedable_rng;
mod from_rng;
mod seed;
mod seedable_rng;
mod weighted_index;
mod xorshift;

use crate::misc::{AsciiGraphic, TryArithmetic};
pub use cha_cha20::ChaCha20;
use core::{cell::Cell, iter, ops::Range};
pub use crypto_rng::CryptoRng;
pub use crypto_seedable_rng::CryptoSeedableRng;
pub use from_rng::FromRng;
pub use seed::*;
pub use seedable_rng::SeedableRng;
pub use weighted_index::*;
pub use xorshift::*;

/// Random number generator.
///
/// Abstraction tailored for the needs of this project.
pub trait Rng: Sized {
  /// Returns an infinite iterator that will always output printable ASCII bytes.
  #[inline]
  fn ascii_graphic_iter(&mut self) -> impl Iterator<Item = AsciiGraphic> {
    iter::repeat_with(|| self.u8_4())
      .flat_map(IntoIterator::into_iter)
      .filter_map(|el| AsciiGraphic::new(el).ok())
  }

  /// Chooses a random element from the slice. Returns `None` if the slice is empty.
  #[inline]
  fn choose_from_slice<'slice, T>(&mut self, slice: &'slice [T]) -> Option<&'slice T> {
    slice.get(usize::from_rng(self).checked_rem(slice.len())?)
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

  /// Picks a random value from the exclusive `range`.
  ///
  /// Returns `None` if the range is empty or the range start is greater or equal to the range end.
  #[inline]
  fn pick_from_range<T>(&mut self, range: Range<T>) -> Option<T>
  where
    T: Clone + From<u8> + FromRng<Self> + PartialOrd + TryArithmetic<Output = T>,
  {
    if range.start >= range.end {
      return None;
    }
    let len = range.end.try_sub(range.start.clone()).ok()?;
    let random = T::from_rng(self);
    let mut offset = random.try_rem(len.clone()).ok()?;
    if random < T::from(0) {
      offset = offset.try_add(len).ok()?;
    }
    range.start.try_add(offset).ok()
  }

  /// Shuffles a mutable slice in place.
  #[inline]
  fn shuffle_slice<T>(&mut self, slice: &mut [T]) {
    let len = slice.len();
    if len <= 1 {
      return;
    }
    for from_idx in 1..len {
      let to_idx = usize::from_rng(self).checked_rem(from_idx.wrapping_add(1)).unwrap_or_default();
      slice.swap(from_idx, to_idx);
    }
  }

  /// Creates an array of 4 bytes.
  fn u8_4(&mut self) -> [u8; 4];

  /// Creates an array of 8 bytes.
  fn u8_8(&mut self) -> [u8; 8];

  /// Creates an array of 16 bytes.
  fn u8_16(&mut self) -> [u8; 16];

  /// Creates an array of 32 bytes.
  fn u8_32(&mut self) -> [u8; 32];
}

impl<T> Rng for Cell<T>
where
  T: Copy + Rng,
{
  #[inline]
  fn u8_4(&mut self) -> [u8; 4] {
    (&*self).u8_4()
  }

  #[inline]
  fn u8_8(&mut self) -> [u8; 8] {
    (&*self).u8_8()
  }

  #[inline]
  fn u8_16(&mut self) -> [u8; 16] {
    (&*self).u8_16()
  }

  #[inline]
  fn u8_32(&mut self) -> [u8; 32] {
    (&*self).u8_32()
  }
}

impl<T> Rng for &Cell<T>
where
  T: Copy + Rng,
{
  #[inline]
  fn u8_4(&mut self) -> [u8; 4] {
    let mut instance = self.get();
    let ret = instance.u8_4();
    self.set(instance);
    ret
  }

  #[inline]
  fn u8_8(&mut self) -> [u8; 8] {
    let mut instance = self.get();
    let ret = instance.u8_8();
    self.set(instance);
    ret
  }

  #[inline]
  fn u8_16(&mut self) -> [u8; 16] {
    let mut instance = self.get();
    let ret = instance.u8_16();
    self.set(instance);
    ret
  }

  #[inline]
  fn u8_32(&mut self) -> [u8; 32] {
    let mut instance = self.get();
    let ret = instance.u8_32();
    self.set(instance);
    ret
  }
}

impl<T> Rng for &mut T
where
  T: Rng,
{
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

  #[inline]
  fn u8_32(&mut self) -> [u8; 32] {
    (*self).u8_32()
  }
}
