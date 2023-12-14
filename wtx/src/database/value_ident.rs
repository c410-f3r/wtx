/// Value Identifier
pub trait ValueIdent<I> {
  /// Underlying index that represents this instance.
  fn idx(&self, input: &I) -> Option<usize>;
}

impl ValueIdent<()> for str {
  #[inline]
  fn idx(&self, _: &()) -> Option<usize> {
    None
  }
}

impl<I, T> ValueIdent<I> for &T
where
  T: ValueIdent<I> + ?Sized,
{
  #[inline]
  fn idx(&self, input: &I) -> Option<usize> {
    (**self).idx(input)
  }
}

impl<I> ValueIdent<I> for () {
  #[inline]
  fn idx(&self, _: &I) -> Option<usize> {
    None
  }
}

impl<I> ValueIdent<I> for usize {
  #[inline]
  fn idx(&self, _: &I) -> Option<usize> {
    Some(*self)
  }
}
