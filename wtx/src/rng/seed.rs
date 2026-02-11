use crate::{calendar::Instant, misc::Usize};
use alloc::boxed::Box;
use core::{panic::Location, ptr};

/// Uses a combination of weak strategies that will likely result in poor results. This method
/// is usually good enough for non-cryptography hashing algorithms.
///
/// 1. The address of a stack and a heap allocation.
/// 2. The line and column of the caller location.
/// 3. The current timestamp, if a provider like `std` or `embassy-time` is active.
/// 4. The random state of `std::hash::RandomState` if `std` is active.
/// 5. The interaction of static state originated from previous operations.
#[inline]
#[track_caller]
pub fn simple_seed() -> u64 {
  let location = Location::caller();
  let (stack, heap) = (7, Box::new(7));
  let mut seed = Usize::from_usize(ptr::addr_of!(stack).addr()).into_u64();
  seed = mix(seed, Usize::from_usize(ptr::addr_of!(heap).addr()).into_u64());
  seed = mix(seed, location.column().into());
  seed = mix(seed, location.line().into());
  if let Ok(timestamp) = Instant::now_timestamp(0).map(|dur| dur.as_nanos()) {
    let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = timestamp.to_le_bytes();
    let lo = u64::from_le_bytes([a, b, c, d, e, f, g, h]);
    let hi = u64::from_le_bytes([i, j, k, l, m, n, o, p]);
    seed = mix(seed, lo);
    seed = mix(seed, hi);
  }
  #[cfg(feature = "std")]
  {
    use core::hash::{BuildHasher, Hasher};
    use std::hash::RandomState;
    seed = mix(seed, Hasher::finish(&BuildHasher::build_hasher(&RandomState::new())));
  }
  #[cfg(not(feature = "std"))]
  {
    use core::sync::atomic::{AtomicU64, Ordering};
    static STATE: AtomicU64 = AtomicU64::new(0);
    seed = mix(seed, STATE.load(Ordering::Relaxed));
    STATE.store(seed, Ordering::Relaxed);
  }
  #[cfg(feature = "std")]
  {
    use core::cell::Cell;
    std::thread_local! {
      static STATE: Cell<u64> = const { Cell::new(0) };
    }
    STATE.with(|cell| {
      seed = mix(seed, cell.get());
      cell.set(seed);
    });
  }
  seed
}

// Credits to the `foldhash` project.
fn mix(seed: u64, number: u64) -> u64 {
  let x = seed ^ number;
  let y = 10_526_836_309_316_205_339u64;
  let array = u128::from(x).wrapping_mul(u128::from(y)).to_le_bytes();
  let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = array;
  let lo = u64::from_le_bytes([a, b, c, d, e, f, g, h]);
  let hi = u64::from_le_bytes([i, j, k, l, m, n, o, p]);
  lo ^ hi
}
