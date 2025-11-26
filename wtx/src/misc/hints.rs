#[cold]
#[inline(never)]
#[track_caller]
pub(crate) const fn _unlikely_elem<T>(elem: T) -> T {
  elem
}

#[allow(clippy::panic, reason = "programming error that should be unreachable")]
#[cold]
#[inline(never)]
#[track_caller]
pub(crate) const fn _unlikely_unreachable() -> ! {
  panic!("Entered in a branch that should be impossible, which is likely a programming error");
}
