//! Random Number Generators

mod cha_cha20;
#[cfg(feature = "chacha20")]
mod chacha20;
mod crypto_rng;
#[cfg(feature = "fastrand")]
mod fastrand;
mod from_rng;
mod seed;
mod seedable_rng;
mod weighted_index;
mod xorshift;

use crate::{misc::TryArithmetic, sync::AtomicCell};
pub use cha_cha20::ChaCha20;
use core::{cell::Cell, iter, ops::Range};
pub use crypto_rng::CryptoRng;
pub use from_rng::FromRng;
pub use seed::*;
pub use seedable_rng::SeedableRng;
pub use weighted_index::*;
pub use xorshift::*;

/// Random number generator.
///
/// Abstraction tailored for the needs of this project.
pub trait Rng
where
  Self: Sized,
{
  /// Returns an infinite iterator that will always output printable ASCII bytes.
  #[inline]
  fn ascii_graphic_iter(&mut self) -> impl Iterator<Item = u8> {
    iter::repeat_with(|| self.u8_4()).flat_map(IntoIterator::into_iter).filter(u8::is_ascii_graphic)
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
  /// Returns `None` if the range is empty or range start is greater or equal to range end.
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

  /// Creates a byte
  fn u8(&mut self) -> u8;

  /// Creates an array of 4 bytes.
  fn u8_4(&mut self) -> [u8; 4];

  /// Creates an array of 8 bytes.
  fn u8_8(&mut self) -> [u8; 8];

  /// Creates an array of 16 bytes.
  fn u8_16(&mut self) -> [u8; 16];

  /// Creates an array of 32 bytes.
  fn u8_32(&mut self) -> [u8; 32];
}

impl<T> Rng for AtomicCell<T>
where
  T: Copy + Eq + Rng,
{
  #[inline]
  fn u8(&mut self) -> u8 {
    (&*self).u8()
  }

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

impl<T> Rng for &AtomicCell<T>
where
  T: Copy + Eq + Rng,
{
  #[inline]
  fn u8(&mut self) -> u8 {
    let mut ret = 0;
    let _rslt = self.update(|mut el| {
      ret = el.u8();
      el
    });
    ret
  }

  #[inline]
  fn u8_4(&mut self) -> [u8; 4] {
    let mut ret = [0; 4];
    let _rslt = self.update(|mut el| {
      ret = el.u8_4();
      el
    });
    ret
  }

  #[inline]
  fn u8_8(&mut self) -> [u8; 8] {
    let mut ret = [0; 8];
    let _rslt = self.update(|mut el| {
      ret = el.u8_8();
      el
    });
    ret
  }

  #[inline]
  fn u8_16(&mut self) -> [u8; 16] {
    let mut ret = [0; 16];
    let _rslt = self.update(|mut el| {
      ret = el.u8_16();
      el
    });
    ret
  }

  #[inline]
  fn u8_32(&mut self) -> [u8; 32] {
    let mut ret = [0; 32];
    let _rslt = self.update(|mut el| {
      ret = el.u8_32();
      el
    });
    ret
  }
}

impl<T> Rng for Cell<T>
where
  T: Copy + Rng,
{
  #[inline]
  fn u8(&mut self) -> u8 {
    (&*self).u8()
  }

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
  fn u8(&mut self) -> u8 {
    let mut instance = self.get();
    let ret = instance.u8();
    self.set(instance);
    ret
  }

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

  #[inline]
  fn u8_32(&mut self) -> [u8; 32] {
    (*self).u8_32()
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    collection::Vector,
    rng::{Rng, Xorshift64},
  };

  #[test]
  fn ascii_graphic_bytes() {
    let mut rng = Xorshift64::from(123);
    let bytes = Vector::from_iterator(rng.ascii_graphic_iter().take(16)).unwrap();
    assert_ne!(&bytes[0..8], &bytes[8..16]);
    for elem in &bytes {
      assert!(elem.is_ascii_graphic());
    }
    assert_ne!(bytes, Vector::from_iterator(rng.ascii_graphic_iter().take(16)).unwrap());
  }
}
