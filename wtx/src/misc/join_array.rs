use core::{
  pin::Pin,
  task::{Context, Poll},
};

/// Joins the result of an array of futures, waiting for them all to complete.
///
/// You should `Box` this structure if size is a concern.
#[must_use = "Futures do nothing unless you await them"]
#[derive(Debug)]
pub struct JoinArray<F, const N: usize>
where
  F: Future,
{
  futures: [F; N],
  outputs: Option<[Option<F::Output>; N]>,
}

impl<F, const N: usize> JoinArray<F, N>
where
  F: Future,
{
  /// Creates a new instance
  #[inline]
  pub const fn new(futures: [F; N]) -> Self {
    Self { futures, outputs: Some([const { None }; N]) }
  }
}

impl<F: Future, const N: usize> Future for JoinArray<F, N>
where
  F: Future,
{
  type Output = [F::Output; N];

  #[inline]
  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    // SAFETY: No fields are moved
    let JoinArray { futures, outputs } = unsafe { self.get_unchecked_mut() };
    let Some(outputs_mut) = outputs else {
      #[expect(
        clippy::panic,
        reason = "Compiler will probably remove this branch as long as the user don't mess up"
      )]
      {
        panic!("Can't poll `JoinArray` again after completion");
      }
    };
    let mut is_finished = true;
    for (future, result) in futures.iter_mut().zip(outputs_mut) {
      if result.is_some() {
        continue;
      }
      // SAFETY: No future is moved
      let pinned = unsafe { Pin::new_unchecked(future) };
      if let Poll::Ready(output) = pinned.poll(cx) {
        *result = Some(output);
      } else {
        is_finished = false;
      }
    }
    if is_finished && let Some(array) = outputs.take() {
      #[expect(clippy::unwrap_used, reason = "Compiler removes this branch")]
      return Poll::Ready(array.map(|el| el.unwrap()));
    }
    Poll::Pending
  }
}

#[cfg(test)]
mod tests {
  use crate::misc::JoinArray;
  use core::future::ready;

  #[wtx::test]
  async fn polls_array() {
    assert_eq!(JoinArray::new([ready(1), ready(2)]).await, [1, 2]);
  }
}
