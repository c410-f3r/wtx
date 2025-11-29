/// Hints to the compiler that a callback is unlikely to occur.
#[cold]
#[inline(always)]
#[track_caller]
pub fn unlikely_cb<F, T>(cb: F) -> T
where
  F: FnOnce() -> T,
{
  cb()
}

#[allow(clippy::panic, reason = "programming error that should be unreachable")]
#[cold]
#[inline(always)]
#[track_caller]
pub(crate) const fn _unlikely_unreachable() -> ! {
  panic!("Entered in a branch that should be impossible, which is likely a programming error");
}
