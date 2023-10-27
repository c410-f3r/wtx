//! Random Number Generators

#[cfg(feature = "std")]
pub use self::std::StdRng;
use core::ptr;

/// Abstraction tailored for the needs of this project. Each implementation should manage how
/// seeds are retrieved as well as how numbers are generated.
pub trait Rng {
  /// Creates an array of 4 bytes.
  fn u8_4(&mut self) -> [u8; 4];

  /// Creates an array of 16 bytes.
  fn u8_16(&mut self) -> [u8; 16];
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
  fn u8_16(&mut self) -> [u8; 16] {
    (*self).u8_16()
  }
}

/// Uses a pre-fixed seed, i.e., it doesn't generate randomness at all.
///
/// The number generation is done using a simple XOR strategy.
///
/// You probably shouldn't use this structure in a production environment.
#[derive(Debug)]
pub struct StaticRng(u64);

impl Rng for StaticRng {
  #[inline]
  fn u8_4(&mut self) -> [u8; 4] {
    xor_u8_4(&mut self.0)
  }

  #[inline]
  fn u8_16(&mut self) -> [u8; 16] {
    xor_u8_16(&mut self.0)
  }
}

impl Default for StaticRng {
  #[inline]
  fn default() -> Self {
    struct Foo {
      _bar: usize,
      _baz: usize,
    }
    let elem = Box::new(Foo { _bar: 1, _baz: 2 });
    // SAFETY: Memory location is not relevant
    let n: usize = unsafe {
      let ref_ptr = ptr::addr_of!(elem);
      *ref_ptr.cast()
    };
    if n == 0 {
      return Self(u64::from_be_bytes([55, 120, 216, 218, 191, 63, 200, 169]));
    }
    #[cfg(target_pointer_width = "16")]
    return Self({
      let [a, b] = n.to_be_bytes();
      u64::from_be_bytes([a, b, 0, 0, 0, 0, 0, 0])
    });
    #[cfg(target_pointer_width = "32")]
    return Self({
      let [a, b, c, d] = n.to_be_bytes();
      u64::from_be_bytes([a, b, c, d, 0, 0, 0, 0])
    });
    #[cfg(target_pointer_width = "64")]
    return Self({
      let [a, b, c, d, e, f, g, h] = n.to_be_bytes();
      u64::from_be_bytes([a, b, c, d, e, f, g, h])
    });
    #[cfg(target_pointer_width = "128")]
    return Self({
      let [a, b, c, d, e, f, g, h, ..] = n.to_be_bytes();
      u64::from_be_bytes([a, b, c, d, e, f, g, h])
    });
  }
}

#[cfg(feature = "rand")]
mod rand {
  use crate::rng::Rng;
  use rand::Rng as _;

  macro_rules! implement {
    ($struct:ty) => {
      impl Rng for $struct {
        #[inline]
        fn u8_4(&mut self) -> [u8; 4] {
          self.gen()
        }

        #[inline]
        fn u8_16(&mut self) -> [u8; 16] {
          self.gen()
        }
      }
    };
  }

  implement!(rand::rngs::mock::StepRng);
  implement!(rand::rngs::SmallRng);
}

#[cfg(feature = "std")]
mod std {
  use crate::rng::{xor_u8_16, xor_u8_4, Rng};
  use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hasher},
  };

  /// Derived from the tools provided by the standard library, uses a simple XOR strategy.
  #[derive(Debug)]
  pub struct StdRng(u64);

  impl Rng for StdRng {
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
}

fn xor_numbers(seed: &mut u64) -> impl Iterator<Item = u64> + '_ {
  core::iter::repeat_with(move || {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    *seed
  })
}

fn xor_u8_4(seed: &mut u64) -> [u8; 4] {
  let [a, b, c, d, ..] = xor_numbers(seed).next().unwrap_or_default().to_be_bytes();
  [a, b, c, d]
}

fn xor_u8_16(seed: &mut u64) -> [u8; 16] {
  let mut iter = xor_numbers(seed);
  let [a, b, c, d, e, f, g, h] = iter.next().unwrap_or_default().to_be_bytes();
  let [i, j, k, l, m, n, o, p] = iter.next().unwrap_or_default().to_be_bytes();
  [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p]
}
