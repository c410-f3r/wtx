/// Used in operations that expand collections.
#[derive(Clone, Copy, Debug)]
pub enum ExpansionTy {
  /// Buffer should be modified with more elements
  Additional(usize),
  /// Buffer should be modified using the exact specified length.
  Len(usize),
}

impl ExpansionTy {
  #[inline]
  pub(crate) fn params(self, len: usize) -> Option<(usize, usize)> {
    Some(match self {
      Self::Additional(elem) => (elem, len.checked_add(elem)?),
      Self::Len(elem) => (elem.checked_sub(len)?, elem),
    })
  }
}
