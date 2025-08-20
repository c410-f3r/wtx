//! Random Number Generators

mod cha_cha20;
mod crypto_rng;
#[cfg(feature = "fastrand")]
mod fastrand;
mod from_rng;
#[cfg(feature = "rand_chacha")]
mod rand_chacha;
mod seed;
mod seedable_rng;
mod xorshift;

use crate::sync::AtomicCell;
pub use cha_cha20::ChaCha20;
use core::{
  cell::Cell,
  iter,
  ops::{Bound, RangeBounds},
};
pub use crypto_rng::CryptoRng;
pub use from_rng::FromRng;
pub use seed::*;
pub use seedable_rng::SeedableRng;
pub use xorshift::*;

/// Abstraction tailored for the needs of this project. Each implementation should manage how
/// seeds are retrieved as well as how numbers are generated.
pub trait Rng
where
  Self: Sized,
{
  /// Returns an infinite iterator that will always output printable ASCII bytes.
  #[inline]
  fn ascii_graphic_iter(&mut self) -> impl Iterator<Item = u8> {
    iter::repeat_with(|| self.u8_8()).flat_map(IntoIterator::into_iter).filter(u8::is_ascii_graphic)
  }

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
}

impl<T> Rng for &AtomicCell<T>
where
  T: Copy + Eq + Rng,
{
  #[inline]
  fn u8(&mut self) -> u8 {
    let mut ret = 0;
    let _rslt = self.fetch_update(|mut el| {
      ret = el.u8();
      Some(el)
    });
    ret
  }

  #[inline]
  fn u8_4(&mut self) -> [u8; 4] {
    let mut ret = [0; 4];
    let _rslt = self.fetch_update(|mut el| {
      ret = el.u8_4();
      Some(el)
    });
    ret
  }

  #[inline]
  fn u8_8(&mut self) -> [u8; 8] {
    let mut ret = [0; 8];
    let _rslt = self.fetch_update(|mut el| {
      ret = el.u8_8();
      Some(el)
    });
    ret
  }

  #[inline]
  fn u8_16(&mut self) -> [u8; 16] {
    let mut ret = [0; 16];
    let _rslt = self.fetch_update(|mut el| {
      ret = el.u8_16();
      Some(el)
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

#[cfg(test)]
mod tests {
  use crate::{
    collection::Vector,
    rng::{Rng, Xorshift64},
  };

  #[test]
  fn ascii_graphic_bytes() {
    let mut rng = Xorshift64::from(123);
    let bytes = Vector::from_iter(rng.ascii_graphic_iter().take(16)).unwrap();
    assert_ne!(&bytes[0..8], &bytes[8..16]);
    for elem in &bytes {
      assert!(elem.is_ascii_graphic());
    }
    assert_ne!(bytes, Vector::from_iter(rng.ascii_graphic_iter().take(16)).unwrap());
  }
}
