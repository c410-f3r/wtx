use crate::{calendar::Instant, misc::Usize};
use alloc::boxed::Box;
use core::{panic::Location, ptr};

/// Uses a combination of weak strategies that will likely result in poor results.
///
/// 1. The address of a heap allocation.
/// 2. The line and column of the caller location.
/// 3. The current duration, if any.
#[inline]
pub fn simple_seed() -> u64 {
  let heap = Box::new(1u8);
  let location = Location::caller();
  let ptr_addr = ptr::addr_of!(heap).addr();
  let mut seed = Usize::from_usize(ptr_addr).into_u64();
  mix_with_time(&mut seed);
  seed = mix(seed, u64::from(location.column().wrapping_add(location.line())));
  seed
}

/// Seed retrieved from the machinery of the standard library.
#[cfg(feature = "std")]
#[inline]
pub fn std_seed() -> u64 {
  use core::hash::{BuildHasher, Hasher};
  use std::hash::RandomState;
  let mut seed = Hasher::finish(&BuildHasher::build_hasher(&RandomState::new()));
  mix_with_time(&mut seed);
  seed
}

// Credits to the `foldhash` project.
fn folded_multiplication(x: u64, y: u64) -> u64 {
  let array = u128::from(x).wrapping_mul(u128::from(y)).to_le_bytes();
  let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = array;
  let hi = u64::from_le_bytes([a, b, c, d, e, f, g, h]);
  let lo = u64::from_le_bytes([i, j, k, l, m, n, o, p]);
  hi ^ lo
}

// Credits to the `foldhash` project.
fn mix(seed: u64, n: u64) -> u64 {
  const FIXED: u64 = 10_526_836_309_316_205_339;
  folded_multiplication(seed ^ n, FIXED)
}

fn mix_with_time(seed: &mut u64) {
  if let Ok(timestamp) = Instant::now_timestamp(0).map(|el| el.as_nanos()) {
    let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = timestamp.to_le_bytes();
    let hi = u64::from_le_bytes([a, b, c, d, e, f, g, h]);
    let lo = u64::from_le_bytes([i, j, k, l, m, n, o, p]);
    *seed = mix(*seed, hi);
    *seed = mix(*seed, lo);
  }
}
