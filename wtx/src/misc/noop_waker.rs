use core::{
  ptr,
  task::{RawWaker, RawWakerVTable, Waker},
};

const VTABLE: RawWakerVTable = RawWakerVTable::new(noop_clone, noop, noop, noop);

// FIXME(STABLE): noop_waker
/// A waker that does nothing.
pub static NOOP_WAKER: Waker = {
  let raw = RawWaker::new(ptr::null(), &VTABLE);
  // SAFETY: Contract is upheld
  unsafe { Waker::from_raw(raw) }
};

unsafe fn noop(_: *const ()) {}

unsafe fn noop_clone(_: *const ()) -> RawWaker {
  RawWaker::new(ptr::null(), &VTABLE)
}
