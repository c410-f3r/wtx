use crate::misc::{Usize, facades::atomic_u64::AtomicU64};
use alloc::boxed::Box;
use core::{panic::Location, ptr, sync::atomic::Ordering};

/// Uses a combination of weak strategies that will likely result in poor results.
///
/// 1. The address of a heap allocation.
/// 2. The value of an ever increasing static counter.
/// 3. The line and column of the caller location.
#[inline]
pub fn simple_seed() -> u64 {
  static COUNTER: AtomicU64 = AtomicU64::new(0);
  let heap = Box::new(0u8);
  let location = Location::caller();
  // FIXME(STABLE): strict_provenance
  // SAFETY: Memory location is not relevant
  let ptr_addr = unsafe { *ptr::addr_of!(heap).cast() };
  let mut seed = Usize::from_usize(ptr_addr).into_u64();
  seed = mix(seed, COUNTER.fetch_add(1, Ordering::Release));
  seed = mix(seed, u64::from(location.column().wrapping_add(location.line())));
  seed
}

/// Seed retrieved from the machinery of the standard library.
///
/// This method is slower than [`simple_seed`] but will probably deliver better results.
#[cfg(feature = "std")]
#[inline]
pub fn std_seed() -> u64 {
  use crate::misc::GenericTime;
  use core::hash::{BuildHasher, Hasher};
  use std::hash::RandomState;
  let mut seed = Hasher::finish(&BuildHasher::build_hasher(&RandomState::new()));
  if let Ok(timestamp) = GenericTime::timestamp().map(|el| el.as_nanos()) {
    let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = timestamp.to_le_bytes();
    let hi = u64::from_le_bytes([a, b, c, d, e, f, g, h]);
    let lo = u64::from_le_bytes([i, j, k, l, m, n, o, p]);
    seed = mix(mix(seed, hi), lo);
  }
  seed
}

// Credits to the `foldhash` project.
#[inline]
fn folded_multiplication(x: u64, y: u64) -> u64 {
  let array = u128::from(x).wrapping_mul(u128::from(y)).to_le_bytes();
  let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = array;
  let hi = u64::from_le_bytes([a, b, c, d, e, f, g, h]);
  let lo = u64::from_le_bytes([i, j, k, l, m, n, o, p]);
  hi ^ lo
}

#[inline]
fn mix(seed: u64, n: u64) -> u64 {
  const FIXED: u64 = 10_526_836_309_316_205_339;
  folded_multiplication(seed ^ n, FIXED)
}
