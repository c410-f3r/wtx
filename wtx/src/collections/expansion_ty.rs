use crate::collections::LinearStorageLen;

/// Used in operations that expand collections.
#[derive(Clone, Copy, Debug)]
pub enum ExpansionTy<L> {
  /// Buffer should be modified with more elements
  Additional(L),
  /// Buffer should be modified using the exact specified length.
  Len(L),
}

impl<L> ExpansionTy<L>
where
  L: LinearStorageLen,
{
  pub(crate) fn params(self, len: L) -> Option<(L, L)> {
    Some(match self {
      Self::Additional(elem) => (elem, len.checked_add(elem)?),
      Self::Len(elem) => (elem.checked_sub(len)?, elem),
    })
  }
}
