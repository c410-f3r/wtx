#[cold]
#[inline(never)]
#[track_caller]
pub(crate) const fn unlikely_elem<T>(elem: T) -> T {
  elem
}

#[cold]
#[inline(never)]
#[track_caller]
pub(crate) const fn _unreachable() -> ! {
  panic!("Entered in a branch that should be impossible, which is likely a programming error");
}
