use core::{
  mem,
  pin::Pin,
  task::{Context, Poll},
};

use crate::collections::ArrayVectorU8;

/// Joins the result of a dynamic array of futures, waiting for them all to complete.
///
/// You should `Box` this structure if size is a concern.
#[must_use = "Futures do nothing unless you await them"]
#[derive(Debug)]
pub struct JoinArrayVector<F, const N: usize>
where
  F: Future,
{
  futures: ArrayVectorU8<F, N>,
  outputs: ArrayVectorU8<Option<F::Output>, N>,
}

impl<F, const N: usize> JoinArrayVector<F, N>
where
  F: Future,
{
  /// Creates a new instance
  #[inline]
  pub const fn new(futures: ArrayVectorU8<F, N>) -> Self {
    // FIXME(STABLE): Fill with `None`
    Self { futures, outputs: ArrayVectorU8::new() }
  }
}

impl<F, const N: usize> Future for JoinArrayVector<F, N>
where
  F: Future,
{
  type Output = ArrayVectorU8<F::Output, N>;

  #[inline]
  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    // SAFETY: No fields are moved
    let Self { futures, outputs } = unsafe { self.get_unchecked_mut() };
    if outputs.is_empty() && !futures.is_empty() {
      for _ in 0..futures.len() {
        let _rslt = outputs.push(None);
      }
    }
    let mut is_finished = true;
    for (future, output_opt) in futures.iter_mut().zip(outputs.iter_mut()) {
      if output_opt.is_some() {
        continue;
      }
      // SAFETY: No future is moved
      let pinned = unsafe { Pin::new_unchecked(future) };
      if let Poll::Ready(output) = pinned.poll(cx) {
        *output_opt = Some(output);
      } else {
        is_finished = false;
      }
    }
    if is_finished {
      let mut results = ArrayVectorU8::new();
      for elem in mem::take(outputs) {
        #[expect(
          clippy::unwrap_used,
          reason = "`is_finished` guarantees that all elements are filled"
        )]
        let _rslt = results.push(elem.unwrap());
      }
      futures.clear();
      return Poll::Ready(results);
    }
    Poll::Pending
  }
}

/// Joins the result of a dynamic array of futures, waiting for them all to complete. However, as
/// soon as one future returns an error, the whole process aborts early.
///
/// You should `Box` this structure if size is a concern.
#[must_use = "Futures do nothing unless you await them"]
#[derive(Debug)]
pub struct TryJoinArrayVector<C, E, F, T, const N: usize>
where
  F: Future<Output = Result<T, E>>,
{
  cb: C,
  futures: ArrayVectorU8<F, N>,
  outputs: ArrayVectorU8<Option<T>, N>,
}

impl<C, E, F, T, const N: usize> TryJoinArrayVector<C, E, F, T, N>
where
  F: Future<Output = Result<T, E>>,
{
  /// Creates a new instance
  #[inline]
  pub const fn new(futures: ArrayVectorU8<F, N>, cb: C) -> Self {
    // FIXME(STABLE): Fill with `None`
    Self { cb, futures, outputs: ArrayVectorU8::new() }
  }
}

impl<C, E, F, T, const N: usize> Future for TryJoinArrayVector<C, E, F, T, N>
where
  C: FnMut(T) -> Result<T, E>,
  F: Future<Output = Result<T, E>>,
{
  type Output = Result<ArrayVectorU8<T, N>, E>;

  #[inline]
  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    // SAFETY: No fields are moved
    let Self { cb, futures, outputs } = unsafe { self.get_unchecked_mut() };
    if outputs.is_empty() && !futures.is_empty() {
      for _ in 0..futures.len() {
        let _rslt = outputs.push(None);
      }
    }
    let mut is_finished = true;
    for (future, output_opt) in futures.iter_mut().zip(outputs.iter_mut()) {
      if output_opt.is_some() {
        continue;
      }
      // SAFETY: No future is moved
      let pinned = unsafe { Pin::new_unchecked(future) };
      if let Poll::Ready(output) = pinned.poll(cx) {
        *output_opt = Some(cb(output?)?);
      } else {
        is_finished = false;
      }
    }
    if is_finished {
      let mut results = ArrayVectorU8::new();
      for elem in mem::take(outputs) {
        #[expect(
          clippy::unwrap_used,
          reason = "`is_finished` guarantees that all elements are filled"
        )]
        let _rslt = results.push(elem.unwrap());
      }
      futures.clear();
      return Poll::Ready(Ok(results));
    }
    Poll::Pending
  }
}

#[cfg(test)]
mod tests {
  use crate::{collections::ArrayVectorU8, misc::join_array_vector::JoinArrayVector};
  use core::{
    pin::Pin,
    task::{Context, Poll},
  };

  #[wtx::test]
  async fn polls_array_vector() {
    let futures = ArrayVectorU8::from([One, One]);
    let fut = &mut JoinArrayVector::new(futures);
    assert_eq!((&mut *fut).await, ArrayVectorU8::from([1, 1]));
    assert_eq!((&mut *fut).await, ArrayVectorU8::new());
    assert_eq!((&mut *fut).await, ArrayVectorU8::new());
  }

  struct One;

  impl Future for One {
    type Output = i32;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
      Poll::Ready(1)
    }
  }
}
