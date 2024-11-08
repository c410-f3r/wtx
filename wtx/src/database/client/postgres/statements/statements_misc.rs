#[derive(Debug)]
pub(crate) struct StatementsMisc {
  pub(crate) columns_len: usize,
  pub(crate) types_len: usize,
}

impl StatementsMisc {
  #[inline]
  pub(crate) fn new(columns_len: usize, types_len: usize) -> Self {
    Self { columns_len, types_len }
  }
}
