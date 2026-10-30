use core::task::Waker;

/// An element of [`crate::misc::waiters::Waiters`].
#[derive(Debug)]
pub(crate) enum Waiter {
  /// No longer referenced by anyone. Can be discarded.
  Removed,
  /// Waiting to be awakened.
  Waiting(Waker),
  /// Already awakened but not yet removed or re-registered.
  Woken,
}

impl Waiter {
  #[inline]
  pub(crate) fn register(
    &mut self,
    waiting_count: &mut usize,
    woken_count: &mut usize,
    waker: &Waker,
  ) {
    match self {
      Self::Removed => {
        *waiting_count = waiting_count.wrapping_add(1);
        *self = Self::Waiting(waker.clone());
      }
      Self::Waiting(elem) => {
        elem.clone_from(waker);
      }
      Self::Woken => {
        *waiting_count = waiting_count.wrapping_add(1);
        *woken_count = woken_count.wrapping_sub(1);
        *self = Self::Waiting(waker.clone());
      }
    }
  }
}
