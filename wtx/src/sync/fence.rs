/// An atomic fence.
///
/// Fences create synchronization between themselves and atomic operations or fences in other
/// threads. To achieve this, a fence prevents the compiler and CPU from reordering certain types of
/// memory operations around it.
#[expect(clippy::disallowed_methods, reason = "this is the only placed")]
#[inline]
pub fn fence(order: core::sync::atomic::Ordering) {
  cfg_select! {
    feature = "portable-atomic" => {
      portable_atomic::fence(order);
    },
    _ => {
      core::sync::atomic::fence(order);
    },
  }
}
