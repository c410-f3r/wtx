// FIXME(stable): iter_intersperse

use core::iter::{Fuse, FusedIterator};

/// An iterator adapter that places a separator between all elements.
#[derive(Clone, Debug)]
pub struct Intersperse<I>
where
  I: Iterator,
  I::Item: Clone,
{
  started: bool,
  separator: I::Item,
  next_item: Option<I::Item>,
  iter: Fuse<I>,
}

impl<I> Intersperse<I>
where
  I: Iterator,
  I::Item: Clone,
{
  /// Creates a new iterator which places a copy of separator between adjacent items of
  /// the original iterator.
  #[inline]
  pub fn new(iter: I, separator: I::Item) -> Self {
    Self { started: false, separator, next_item: None, iter: iter.fuse() }
  }
}

impl<I> FusedIterator for Intersperse<I>
where
  I: FusedIterator,
  I::Item: Clone,
{
}

impl<I> Iterator for Intersperse<I>
where
  I: Iterator,
  I::Item: Clone,
{
  type Item = I::Item;

  #[inline]
  fn fold<B, F>(self, init: B, f: F) -> B
  where
    Self: Sized,
    F: FnMut(B, Self::Item) -> B,
  {
    let separator = self.separator;
    intersperse_fold(self.iter, init, f, move || separator.clone(), self.started, self.next_item)
  }

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    if self.started {
      if let Some(v) = self.next_item.take() {
        Some(v)
      } else {
        let next_item = self.iter.next();
        next_item.is_some().then(|| {
          self.next_item = next_item;
          self.separator.clone()
        })
      }
    } else {
      self.started = true;
      self.iter.next()
    }
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    intersperse_size_hint(&self.iter, self.started, self.next_item.is_some())
  }
}

#[inline]
fn intersperse_fold<I, B, F, G>(
  mut iter: I,
  init: B,
  mut f: F,
  mut separator: G,
  started: bool,
  mut next_item: Option<I::Item>,
) -> B
where
  I: Iterator,
  F: FnMut(B, I::Item) -> B,
  G: FnMut() -> I::Item,
{
  let mut accum = init;

  let first = if started { next_item.take() } else { iter.next() };
  if let Some(x) = first {
    accum = f(accum, x);
  }

  iter.fold(accum, |mut elem, x| {
    elem = f(elem, separator());
    elem = f(elem, x);
    elem
  })
}

#[inline]
fn intersperse_size_hint<I>(iter: &I, started: bool, next_is_some: bool) -> (usize, Option<usize>)
where
  I: Iterator,
{
  let (lo, hi) = iter.size_hint();
  (
    lo.saturating_sub((!started).into()).saturating_add(next_is_some.into()).saturating_add(lo),
    hi.and_then(|elem| {
      elem.saturating_sub((!started).into()).saturating_add(next_is_some.into()).checked_add(elem)
    }),
  )
}
