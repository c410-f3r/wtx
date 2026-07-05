use crate::{calendar::Instant, misc::Usize};
use alloc::boxed::Box;
use core::{panic::Location, ptr};

const BEGIN: u64 = 5_871_781_006_564_002_453;
const MIX: u64 = 10_526_836_309_316_205_339;

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
  let mut seed = BEGIN;
  seed = mix(seed, Usize::from_usize(ptr::addr_of!(stack).addr()).into_u64());
  seed = mix(seed, Usize::from_usize(ptr::addr_of!(heap).addr()).into_u64());
  seed = mix(seed, location.column().into());
  seed = mix(seed, location.line().into());
  if let Ok(timestamp) = Instant::now_timestamp().map(|dur| dur.as_nanos()) {
    let [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15] =
      timestamp.to_le_bytes();
    let lo = u64::from_le_bytes([b0, b1, b2, b3, b4, b5, b6, b7]);
    let hi = u64::from_le_bytes([b8, b9, b10, b11, b12, b13, b14, b15]);
    seed = mix(seed, lo);
    seed = mix(seed, hi);
  }
  cfg_select! {
    feature = "std" => {
      use core::{
        cell::Cell,
        hash::{BuildHasher, Hasher}
      };
      use std::hash::RandomState;

      std::thread_local! {
        static STATE: Cell<u64> = const { Cell::new(1) };
      }

      STATE.with(|cell| {
        seed = mix(seed, cell.get());
        cell.set(seed);
      });
      seed = mix(seed, Hasher::finish(&BuildHasher::build_hasher(&RandomState::new())));
    }
    _ => {
      use core::sync::atomic::{AtomicU64, Ordering};
      static STATE: AtomicU64 = AtomicU64::new(1);
      seed = mix(seed, STATE.load(Ordering::Relaxed));
      STATE.store(seed, Ordering::Relaxed);
    }
  }
  seed
}

// Credits to the `foldhash` project.
fn mix(seed: u64, number: u64) -> u64 {
  let x = seed ^ number;
  let y = MIX;
  let array = u128::from(x).wrapping_mul(u128::from(y)).to_le_bytes();
  let [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15] = array;
  let lo = u64::from_le_bytes([b0, b1, b2, b3, b4, b5, b6, b7]);
  let hi = u64::from_le_bytes([b8, b9, b10, b11, b12, b13, b14, b15]);
  lo ^ hi
}
