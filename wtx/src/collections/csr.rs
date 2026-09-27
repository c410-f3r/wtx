use crate::collections::{LinearStorageLen, Vector};
use core::{hint::cold_path, range::Range};

/// [`Csr`] with a capacity limited by `u8`.
pub type CsrU8<T> = Csr<u8, T>;
/// [`Csr`] with a capacity limited by `u16`.
pub type CsrU16<T> = Csr<u16, T>;
/// [`Csr`] with a capacity limited by `u32`.
pub type CsrU32<T> = Csr<u32, T>;
/// [`Csr`] with a capacity limited by `usize`.
pub type CsrUsize<T> = Csr<usize, T>;

/// Errors of [Csr].
#[derive(Clone, Copy, Debug)]
pub enum CsrError {
  #[doc = doc_reserve_overflow!()]
  CapacityOverflow,
  /// When constructing a line, columns must be provided in ascending order.
  NonAscendingColumnInLineBuilder,
}

/// Compressed Sparse Row
#[derive(Debug)]
pub struct Csr<L, T>
where
  L: LinearStorageLen,
{
  columns_indices: Vector<L, usize>,
  columns_len: L,
  data: Vector<L, T>,
  rows_offsets: Vector<L, usize>,
}

impl<L, T> Csr<L, T>
where
  L: LinearStorageLen,
{
  /// New empty instance
  #[inline]
  pub const fn new() -> Self {
    Self {
      columns_indices: Vector::new(),
      columns_len: L::ZERO,
      data: Vector::new(),
      rows_offsets: Vector::new(),
    }
  }

  /// Number of columns
  #[inline]
  pub fn columns(&self) -> L {
    self.columns_len
  }

  /// Returns the data of a given coordinate, if any.
  #[inline]
  pub fn get(&self, row_idx: usize, column_idx: usize) -> Option<&T> {
    let row_range = self.row_range(row_idx)?;
    // SAFETY: Constructors always make indices point to valid references
    let row_columns_indices = unsafe { self.columns_indices.get(row_range).unwrap_unchecked() };
    if let Ok(idx) = row_columns_indices.binary_search(&column_idx) {
      // SAFETY: Constructors always make indices point to valid references
      let row_data = unsafe { self.data.get(row_range).unwrap_unchecked() };
      // SAFETY: A column position is always associated with a data position
      return Some(unsafe { row_data.get(idx).unwrap_unchecked() });
    }
    None
  }

  /// See [`CsrLineBuilder`].
  #[inline]
  pub fn line_builder(&mut self) -> crate::Result<CsrLineBuilder<'_, L, T>> {
    if self.rows_offsets.remaining_capacity_max() == L::ZERO {
      return Err(CsrError::CapacityOverflow.into());
    }
    Ok(CsrLineBuilder { csr: self, inserted_columns: 0, last_column_idx: 0 })
  }

  /// Iterates over [`Csr`] rows
  #[inline]
  pub fn row_iter(&self) -> CsrRowIter<'_, L, T> {
    CsrRowIter { columns_indices_prev: 0, csr: self, row_idx: 0 }
  }

  /// Number of rows
  #[inline]
  pub fn rows(&self) -> L {
    self.rows_offsets.len()
  }

  #[inline(always)]
  fn row_range(&self, row_idx: usize) -> Option<Range<usize>> {
    match self.rows_offsets.get(..=row_idx) {
      Some([.., begin, end]) => Some((*begin..*end).into()),
      Some([end]) => {
        cold_path();
        Some((0..*end).into())
      }
      _ => {
        cold_path();
        None
      }
    }
  }
}

impl<L, T> Default for Csr<L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

/// Pushes data into a line that is finished when dropped.
#[derive(Debug)]
pub struct CsrLineBuilder<'csr, L, T>
where
  L: LinearStorageLen,
{
  csr: &'csr mut Csr<L, T>,
  inserted_columns: usize,
  last_column_idx: usize,
}

impl<L, T> CsrLineBuilder<'_, L, T>
where
  L: LinearStorageLen,
{
  /// Pushes `data` into the current line that is being constructed.
  #[inline]
  pub fn push_data(&mut self, column_idx: usize, data: T) -> crate::Result<()> {
    if self.inserted_columns > 0 && column_idx <= self.last_column_idx {
      cold_path();
      return Err(CsrError::NonAscendingColumnInLineBuilder.into());
    }
    if self.csr.columns_indices.remaining_capacity_max() == L::ZERO
      || self.csr.data.remaining_capacity_max() == L::ZERO
    {
      cold_path();
      return Err(CsrError::CapacityOverflow.into());
    }
    let Some(required_len) = column_idx.checked_add(1) else {
      cold_path();
      return Err(CsrError::CapacityOverflow.into());
    };
    self.csr.columns_len = self.csr.columns_len.max(L::from_usize(required_len)?);
    let _rslt0 = self.csr.columns_indices.push(column_idx);
    let _rslt1 = self.csr.data.push(data);
    self.inserted_columns = self.inserted_columns.wrapping_add(1);
    self.last_column_idx = column_idx;
    Ok(())
  }
}

impl<L, T> Drop for CsrLineBuilder<'_, L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn drop(&mut self) {
    let prev = self.csr.rows_offsets.last().copied().unwrap_or(0);
    let rslt = self.csr.rows_offsets.push(prev.wrapping_add(self.inserted_columns));
    // Will never fail because the only constructor of this type always checks if there is capacity.
    debug_assert!(rslt.is_ok());
  }
}

/// A [`Csr`] row
#[derive(Debug)]
pub struct CsrRow<'csr, T> {
  columns_indices: &'csr [usize],
  data: &'csr [T],
}

impl<'csr, T> CsrRow<'csr, T> {
  /// Column indices.
  #[inline]
  pub fn columns_indices(&self) -> &'csr [usize] {
    self.columns_indices
  }

  /// All data contained in this row.
  #[inline]
  pub fn data(&self) -> &'csr [T] {
    self.data
  }

  /// Returns the data of a given column, if any.
  #[inline]
  pub fn get(&self, column_idx: usize) -> Option<&'csr T> {
    if let Ok(idx) = self.columns_indices.binary_search(&column_idx) {
      // SAFETY: A column position is always associated with a data position
      return Some(unsafe { self.data.get(idx).unwrap_unchecked() });
    }
    None
  }

  /// Iterates over the data that composes this row.
  #[inline]
  pub fn iter(&self) -> impl Iterator<Item = (usize, &'csr T)> {
    self.columns_indices.iter().copied().zip(self.data)
  }
}

/// Iterates over [`Csr`] rows
#[derive(Debug)]
pub struct CsrRowIter<'csr, L, T>
where
  L: LinearStorageLen,
{
  columns_indices_prev: usize,
  csr: &'csr Csr<L, T>,
  row_idx: usize,
}

impl<'csr, L, T> Iterator for CsrRowIter<'csr, L, T>
where
  L: LinearStorageLen,
{
  type Item = CsrRow<'csr, T>;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    let end = self.csr.rows_offsets.get(self.row_idx)?;
    self.row_idx = self.row_idx.wrapping_add(1);
    let row_range: Range<usize> = (self.columns_indices_prev..*end).into();
    self.columns_indices_prev = *end;
    // SAFETY: Constructors always make indices point to valid references
    let columns_indices = unsafe { self.csr.columns_indices.get(row_range).unwrap_unchecked() };
    // SAFETY: Constructors always make indices point to valid references
    let data = unsafe { self.csr.data.get(row_range).unwrap_unchecked() };
    Some(CsrRow { columns_indices, data })
  }
}

#[cfg(test)]
mod tests {
  use crate::collections::CsrU8;

  #[test]
  fn get() {
    let mut csr = CsrU8::new();
    {
      let mut builder = csr.line_builder().unwrap();
      builder.push_data(0, "a").unwrap();
      builder.push_data(2, "b").unwrap();
    }
    {
      let mut builder = csr.line_builder().unwrap();
      builder.push_data(1, "c").unwrap();
      builder.push_data(3, "d").unwrap();
    }

    {
      assert_eq!(csr.get(0, 0), Some(&"a"));
      assert_eq!(csr.get(0, 1), None);
      assert_eq!(csr.get(0, 2), Some(&"b"));
      assert_eq!(csr.get(0, 3), None);
    }
    {
      assert_eq!(csr.get(1, 0), None);
      assert_eq!(csr.get(1, 1), Some(&"c"));
      assert_eq!(csr.get(1, 2), None);
      assert_eq!(csr.get(1, 3), Some(&"d"));
    }
    assert_eq!(csr.get(2, 0), None);
  }

  #[test]
  fn iter() {
    let mut csr = CsrU8::new();
    {
      let mut builder = csr.line_builder().unwrap();
      builder.push_data(1, 10).unwrap();
      builder.push_data(4, 20).unwrap();
    }
    {
      let mut _builder = csr.line_builder().unwrap();
    }
    {
      let mut builder = csr.line_builder().unwrap();
      builder.push_data(0, 30).unwrap();
    }

    let mut rows = csr.row_iter();
    {
      let row = rows.next().unwrap();
      let mut iter = row.iter();
      assert_eq!(iter.next().unwrap(), (1, &10));
      assert_eq!(iter.next().unwrap(), (4, &20));
      assert_eq!(iter.next(), None);
    }
    assert_eq!(rows.next().unwrap().data().len(), 0);
    {
      let row = rows.next().unwrap();
      let mut iter = row.iter();
      assert_eq!(iter.next().unwrap(), (0, &30));
      assert_eq!(iter.next(), None);
    }
    assert!(rows.next().is_none());
  }
}
