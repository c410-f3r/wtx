/// An atomic fence.
#[expect(clippy::disallowed_methods, reason = "this is the only placed")]
#[inline]
pub fn fence(order: crate::sync::Ordering) {
  #[cfg(feature = "portable-atomic")]
  return portable_atomic::fence(order);
  #[cfg(all(feature = "loom", not(any(feature = "portable-atomic"))))]
  return loom::sync::atomic::fence(order);
  #[cfg(not(any(feature = "loom", feature = "portable-atomic")))]
  return core::sync::atomic::fence(order);
}
