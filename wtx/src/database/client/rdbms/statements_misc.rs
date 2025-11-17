#[derive(Debug)]
pub(crate) struct StatementsMisc<A> {
  pub(crate) _aux: A,
  pub(crate) columns_len: usize,
  pub(crate) rows_len: usize,
  pub(crate) types_len: usize,
}

impl<A> StatementsMisc<A> {
  pub(crate) const fn new(aux: A, columns_len: usize, rows_len: usize, types_len: usize) -> Self {
    Self { _aux: aux, columns_len, rows_len, types_len }
  }
}
