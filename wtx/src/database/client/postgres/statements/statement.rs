use crate::database::client::postgres::{statements::column::Column, Ty};

/// ```sql
/// SELECT a,b,c,d FROM table WHERE e = $1 AND f = $2
/// ```
///
/// The columns are "a", "b", "c", "d" and the types are "$1" and "$2".
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Statement<'stmts> {
  columns_len: usize,
  tys_len: usize,
  values: &'stmts [(Column, Ty)],
}

impl<'stmts> Statement<'stmts> {
  #[inline]
  pub(crate) const fn new(
    columns_len: usize,
    tys_len: usize,
    values: &'stmts [(Column, Ty)],
  ) -> Self {
    Self { columns_len, tys_len, values }
  }

  #[inline]
  pub(crate) fn column(&self, idx: usize) -> Option<&Column> {
    let columns = self.values.get(..self.columns_len)?;
    Some(&columns.get(idx)?.0)
  }

  #[inline]
  pub(crate) fn columns(&self) -> impl Iterator<Item = &Column> {
    let columns = self.values.get(..self.columns_len).unwrap_or_default();
    columns.iter().map(|el| &el.0)
  }

  #[cfg(test)]
  #[inline]
  pub(crate) fn ty(&self, idx: usize) -> Option<Ty> {
    Some(self.values.get(..self.tys_len)?.get(idx)?.1)
  }

  #[cfg(test)]
  #[inline]
  pub(crate) fn tys(&self) -> impl Iterator<Item = &Ty> {
    self.values.get(..self.tys_len).unwrap_or_default().iter().map(|el| &el.1)
  }
}
