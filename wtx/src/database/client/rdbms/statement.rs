/// ```sql
/// SELECT a,b,c,d FROM table WHERE a = $1 AND b = $2
/// ```
///
/// The columns are "a", "b", "c", "d" and the types are "$1" and "$2".
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Statement<'stmts, A, C, T> {
  pub(crate) aux: A,
  pub(crate) columns_len: usize,
  pub(crate) rows_len: usize,
  pub(crate) tys_len: usize,
  pub(crate) values: &'stmts [(C, T)],
}

impl<'stmts, A, C, T> Statement<'stmts, A, C, T> {
  pub(crate) const fn new(
    aux: A,
    columns_len: usize,
    rows_len: usize,
    tys_len: usize,
    values: &'stmts [(C, T)],
  ) -> Self {
    Self { aux, columns_len, rows_len, tys_len, values }
  }

  pub(crate) fn column(&self, idx: usize) -> Option<&C> {
    let columns = self.values.get(..self.columns_len)?;
    Some(&columns.get(idx)?.0)
  }

  pub(crate) fn columns(&self) -> &[(C, T)] {
    self.values.get(..self.columns_len).unwrap_or_default()
  }

  #[cfg(test)]
  pub(crate) fn ty(&self, idx: usize) -> Option<&T> {
    Some(&self.values.get(..self.tys_len)?.get(idx)?.1)
  }

  #[cfg(test)]
  pub(crate) fn tys(&self) -> &[(C, T)] {
    self.values.get(..self.tys_len).unwrap_or_default()
  }
}

impl<A, C, T> Default for Statement<'_, A, C, T>
where
  A: Default,
{
  #[inline]
  fn default() -> Self {
    Self { aux: A::default(), columns_len: 0, rows_len: 0, tys_len: 0, values: &[] }
  }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct StatementMut<'stmts, A, C, T> {
  pub(crate) aux: A,
  pub(crate) columns_len: &'stmts mut usize,
  pub(crate) rows_len: &'stmts mut usize,
  pub(crate) tys_len: &'stmts mut usize,
  pub(crate) values: &'stmts mut [(C, T)],
}

impl<'stmts, A, C, T> StatementMut<'stmts, A, C, T> {
  pub(crate) const fn new(
    aux: A,
    columns_len: &'stmts mut usize,
    rows_len: &'stmts mut usize,
    tys_len: &'stmts mut usize,
    values: &'stmts mut [(C, T)],
  ) -> Self {
    Self { aux, columns_len, rows_len, tys_len, values }
  }

  pub(crate) fn into_stmt(self) -> Statement<'stmts, A, C, T> {
    Statement::new(self.aux, *self.columns_len, *self.rows_len, *self.tys_len, self.values)
  }

  pub(crate) fn stmt(&self) -> Statement<'_, A, C, T>
  where
    A: Clone,
  {
    Statement::new(self.aux.clone(), *self.columns_len, *self.rows_len, *self.tys_len, self.values)
  }

  #[cfg(feature = "mysql")]
  pub(crate) fn tys(&self) -> &[(C, T)] {
    self.values.get(..*self.tys_len).unwrap_or_default()
  }

  #[cfg(feature = "mysql")]
  pub(crate) fn tys_mut(&mut self) -> &mut [(C, T)] {
    self.values.get_mut(..*self.tys_len).unwrap_or_default()
  }
}
