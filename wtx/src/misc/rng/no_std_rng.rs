use crate::misc::{
  rng::{xor_u8, xor_u8_16, xor_u8_4, xor_u8_8},
  Rng, Usize,
};
use alloc::boxed::Box;
use core::{
  panic::Location,
  ptr,
  sync::atomic::{AtomicU64, Ordering},
};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Uses a combination of weak strategies that will likely result in poor results.
///
/// 1. The address of a heap allocation.
/// 2. A large fixed number.
/// 3. The value of an ever increasing static counter.
/// 4. The line and column of the caller location.
#[derive(Clone, Copy, Debug)]
pub struct NoStdRng(u64);

impl Rng for NoStdRng {
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

impl Default for NoStdRng {
  #[inline]
  #[track_caller]
  fn default() -> Self {
    let elem = Box::new(Foo { _bar: 1, _baz: 2 });
    let ref_ptr = ptr::addr_of!(elem);
    let mut n = Usize::from_usize(ref_ptr.addr()).into_u64();
    n = n.wrapping_add(11_400_714_819_323_198_485);
    n = n.wrapping_add(COUNTER.fetch_add(3, Ordering::Release));
    let location = Location::caller();
    n ^= n << (u64::from(location.column().wrapping_add(location.line())) % 17);
    Self(n)
  }
}

struct Foo {
  _bar: usize,
  _baz: usize,
}
