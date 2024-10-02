use crate::misc::Usize;
use alloc::boxed::Box;
use core::{
  panic::Location,
  ptr,
  sync::atomic::{AtomicU64, Ordering},
};

/// Uses a combination of weak strategies that will likely result in poor results.
///
/// 1. The address of a heap allocation.
/// 2. A large fixed number.
/// 3. The value of an ever increasing static counter.
/// 4. The line and column of the caller location.
#[inline]
pub fn simple_seed() -> u64 {
  static COUNTER: AtomicU64 = AtomicU64::new(0);

  struct Foo {
    _bar: usize,
    _baz: usize,
  }

  let elem = Box::new(Foo { _bar: 1, _baz: 2 });
  // FIXME(STABLE): strict_provenance
  // SAFETY: Memory location is not relevant
  let ptr_addr = unsafe { *ptr::addr_of!(elem).cast() };
  let mut rslt = Usize::from_usize(ptr_addr).into_u64();
  rslt = rslt.wrapping_add(11_400_714_819_323_198_485);
  rslt = rslt.wrapping_add(COUNTER.fetch_add(1, Ordering::Release));
  let location = Location::caller();
  rslt = rslt.wrapping_add(u64::from(location.column().wrapping_add(location.line())));
  rslt
}

/// Seed retrieved from the machinery of the standard library.
#[cfg(feature = "std")]
#[inline]
pub fn std_seed() -> u64 {
  use core::hash::{BuildHasher, Hasher};
  use std::hash::RandomState;
  Hasher::finish(&BuildHasher::build_hasher(&RandomState::new()))
}
