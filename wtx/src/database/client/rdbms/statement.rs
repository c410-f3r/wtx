/// ```sql
/// SELECT a,b,c,d FROM table WHERE e = $1 AND f = $2
/// ```
///
/// The columns are "a", "b", "c", "d" and the types are "$1" and "$2".
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Statement<'stmts, A, C, T> {
  aux: A,
  columns_len: usize,
  tys_len: usize,
  values: &'stmts [(C, T)],
}

impl<'stmts, A, C, T> Statement<'stmts, A, C, T> {
  #[inline]
  pub(crate) const fn new(
    aux: A,
    columns_len: usize,
    tys_len: usize,
    values: &'stmts [(C, T)],
  ) -> Self {
    Self { aux, columns_len, tys_len, values }
  }

  #[inline]
  pub(crate) fn _aux(&self) -> &A {
    &self.aux
  }

  #[inline]
  pub(crate) fn _column(&self, idx: usize) -> Option<&C> {
    let columns = self.values.get(..self.columns_len)?;
    Some(&columns.get(idx)?.0)
  }

  #[inline]
  pub(crate) fn _columns(&self) -> impl Iterator<Item = &C> {
    let columns = self.values.get(..self.columns_len).unwrap_or_default();
    columns.iter().map(|el| &el.0)
  }

  #[inline]
  pub(crate) fn _columns_len(&self) -> usize {
    self.columns_len
  }

  #[cfg(test)]
  #[inline]
  pub(crate) fn _ty(&self, idx: usize) -> Option<&T> {
    Some(&self.values.get(..self.tys_len)?.get(idx)?.1)
  }

  #[inline]
  pub(crate) fn _tys(&self) -> impl Iterator<Item = &T> {
    self.values.get(..self.tys_len).unwrap_or_default().iter().map(|el| &el.1)
  }

  #[inline]
  pub(crate) fn _tys_len(&self) -> usize {
    self.tys_len
  }

  #[inline]
  pub(crate) fn _values(&self) -> &[(C, T)] {
    &self.values
  }
}

impl<'stmts, A, C, T> Default for Statement<'stmts, A, C, T>
where
  A: Default,
{
  #[inline]
  fn default() -> Self {
    Self { aux: A::default(), columns_len: 0, tys_len: 0, values: &[] }
  }
}
