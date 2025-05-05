/// ```sql
/// SELECT a,b,c,d FROM table WHERE e = $1 AND f = $2
/// ```
///
/// The columns are "a", "b", "c", "d" and the types are "$1" and "$2".
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Statement<'stmts, A, C, T> {
  pub(crate) aux: A,
  pub(crate) columns_len: usize,
  pub(crate) tys_len: usize,
  pub(crate) values: &'stmts [(C, T)],
}

impl<'stmts, A, C, T> Statement<'stmts, A, C, T> {
  pub(crate) const fn new(
    aux: A,
    columns_len: usize,
    tys_len: usize,
    values: &'stmts [(C, T)],
  ) -> Self {
    Self { aux, columns_len, tys_len, values }
  }

  pub(crate) fn column(&self, idx: usize) -> Option<&C> {
    let columns = self.values.get(..self.columns_len)?;
    Some(&columns.get(idx)?.0)
  }

  pub(crate) fn columns(&self) -> impl Iterator<Item = &C> {
    let columns = self.values.get(..self.columns_len).unwrap_or_default();
    columns.iter().map(|el| &el.0)
  }

  #[cfg(all(feature = "_async-tests", test))]
  pub(crate) fn ty(&self, idx: usize) -> Option<&T> {
    Some(&self.values.get(..self.tys_len)?.get(idx)?.1)
  }

  #[cfg(feature = "mysql")]
  pub(crate) fn tys(&self) -> impl Iterator<Item = &T> {
    self.values.get(..self.tys_len).unwrap_or_default().iter().map(|el| &el.1)
  }
}

impl<A, C, T> Default for Statement<'_, A, C, T>
where
  A: Default,
{
  #[inline]
  fn default() -> Self {
    Self { aux: A::default(), columns_len: 0, tys_len: 0, values: &mut [] }
  }
}

impl<'stmts, A, C, T> From<StatementMut<'stmts, A, C, T>> for Statement<'stmts, A, C, T> {
  #[inline]
  fn from(from: StatementMut<'stmts, A, C, T>) -> Self {
    Self::new(from.aux, *from.columns_len, *from.tys_len, from.values)
  }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct StatementMut<'stmts, A, C, T> {
  pub(crate) aux: A,
  pub(crate) columns_len: &'stmts mut usize,
  pub(crate) tys_len: &'stmts mut usize,
  pub(crate) values: &'stmts mut [(C, T)],
}

impl<'stmts, A, C, T> StatementMut<'stmts, A, C, T> {
  pub(crate) const fn new(
    aux: A,
    columns_len: &'stmts mut usize,
    tys_len: &'stmts mut usize,
    values: &'stmts mut [(C, T)],
  ) -> Self {
    Self { aux, columns_len, tys_len, values }
  }
}
