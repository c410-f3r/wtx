/// An atomic fence.
///
/// Fences create synchronization between themselves and atomic operations or fences in other
/// threads. To achieve this, a fence prevents the compiler and CPU from reordering certain types of
/// memory operations around it.
#[expect(clippy::disallowed_methods, reason = "this is the only placed")]
#[inline]
pub fn fence(order: core::sync::atomic::Ordering) {
  #[cfg(feature = "portable-atomic")]
  return portable_atomic::fence(order);
  #[cfg(all(feature = "loom", not(any(feature = "portable-atomic"))))]
  return loom::sync::atomic::fence(order);
  #[cfg(not(any(feature = "loom", feature = "portable-atomic")))]
  return core::sync::atomic::fence(order);
}
