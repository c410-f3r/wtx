//! Random Number Generators

#[cfg(feature = "fastrand")]
mod fastrand;
#[cfg(feature = "rand")]
mod rand;
#[cfg(feature = "std")]
mod std;

#[cfg(feature = "std")]
pub use self::std::StdRng;
use alloc::boxed::Box;
use core::ptr;

/// Abstraction tailored for the needs of this project. Each implementation should manage how
/// seeds are retrieved as well as how numbers are generated.
pub trait Rng {
  /// Creates an byte
  fn u8(&mut self) -> u8;

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
  fn u8(&mut self) -> u8 {
    (*self).u8()
  }

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

impl Default for StaticRng {
  #[inline]
  fn default() -> Self {
    struct Foo {
      _bar: usize,
      _baz: usize,
    }
    let elem = Box::new(Foo { _bar: 1, _baz: 2 });
    let ref_ptr = ptr::addr_of!(elem).cast();
    // SAFETY: Memory validation is not relevant
    #[allow(unsafe_code)]
    let n: usize = unsafe { *ref_ptr };
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

fn xor_numbers(seed: &mut u64) -> u64 {
  *seed ^= *seed << 13;
  *seed ^= *seed >> 17;
  *seed ^= *seed << 5;
  *seed
}

fn xor_u8(seed: &mut u64) -> u8 {
  let [a, ..] = xor_numbers(seed).to_be_bytes();
  a
}

fn xor_u8_4(seed: &mut u64) -> [u8; 4] {
  let [a, b, c, d, ..] = xor_numbers(seed).to_be_bytes();
  [a, b, c, d]
}

fn xor_u8_16(seed: &mut u64) -> [u8; 16] {
  let [a, b, c, d, e, f, g, h] = xor_numbers(seed).to_be_bytes();
  let [i, j, k, l, m, n, o, p] = xor_numbers(seed).to_be_bytes();
  [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p]
}
