use core::{
  fmt::{Debug, Formatter},
  future::Future,
  pin::Pin,
  task::{Context, Poll},
};

/// Pools a future in only one try.
pub struct PollOnce<F>(
  /// Future
  pub F,
);

impl<F> Debug for PollOnce<F> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
    f.debug_tuple("PollOnce").finish()
  }
}

impl<F> Future for PollOnce<F>
where
  F: Future + Unpin,
{
  type Output = Option<F::Output>;

  #[inline]
  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    match Pin::new(&mut self.0).poll(cx) {
      Poll::Ready(t) => Poll::Ready(Some(t)),
      Poll::Pending => Poll::Ready(None),
    }
  }
}
