// FIXME(STABLE): iter_intersperse

use core::iter::{Fuse, FusedIterator};

/// An iterator adapter that places a separator between all elements.
#[derive(Clone, Debug)]
pub struct Intersperse<I, S>
where
  I: Iterator,
{
  started: bool,
  separator: S,
  next_item: Option<I::Item>,
  iter: Fuse<I>,
}

impl<I, S> Intersperse<I, S>
where
  I: Iterator,
  I::Item: Clone,
{
  /// Creates a new iterator which places a copy of separator between adjacent items of
  /// the original iterator.
  #[inline]
  pub fn new(iter: impl IntoIterator<IntoIter = I>, separator: S) -> Self {
    Self { started: false, separator, next_item: None, iter: iter.into_iter().fuse() }
  }
}

impl<I, S> FusedIterator for Intersperse<I, S>
where
  I: FusedIterator,
  S: FnMut() -> I::Item,
{
}

impl<I, S> Iterator for Intersperse<I, S>
where
  I: Iterator,
  S: FnMut() -> I::Item,
{
  type Item = I::Item;

  #[inline]
  fn fold<B, F>(self, init: B, f: F) -> B
  where
    Self: Sized,
    F: FnMut(B, Self::Item) -> B,
  {
    intersperse_fold(self.iter, init, f, self.separator, self.started, self.next_item)
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
          (self.separator)()
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

fn intersperse_fold<I, B, F, S>(
  mut iter: I,
  init: B,
  mut f: F,
  mut separator: S,
  started: bool,
  mut next_item: Option<I::Item>,
) -> B
where
  I: Iterator,
  F: FnMut(B, I::Item) -> B,
  S: FnMut() -> I::Item,
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

#[cfg(test)]
mod tests {
  use crate::{collection::Vector, misc::Intersperse};

  #[test]
  fn interspace() {
    assert_eq!(
      Vector::from_iter(Intersperse::new(['0', '1', '2'], || ',')).unwrap().as_slice(),
      &['0', ',', '1', ',', '2']
    );
  }
}
