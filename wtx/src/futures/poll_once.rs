use core::{
  fmt::{Debug, Formatter},
  pin::Pin,
  task::{Context, Poll},
};

/// Polls a future once.
#[must_use = "futures do nothing without .await calls"]
pub struct PollOnce<F> {
  fut: F,
}

impl<F> PollOnce<F> {
  /// New instance
  #[inline]
  pub const fn new(fut: F) -> Self {
    Self { fut }
  }
}

impl<F> Debug for PollOnce<F> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("PollOnce").finish()
  }
}

impl<T, F> Future for PollOnce<F>
where
  F: Future<Output = T> + Unpin,
{
  type Output = Option<T>;

  #[inline]
  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    match Pin::new(&mut self.fut).poll(cx) {
      Poll::Ready(elem) => Poll::Ready(Some(elem)),
      Poll::Pending => Poll::Ready(None),
    }
  }
}
