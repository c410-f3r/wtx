use crate::collection::Vector;

/// A heap-allocated matrix where each row is a contiguous block of variable size.
///
/// Can be seen as a middle ground between dense matrices and CSRs.
pub struct CompressedRows<T> {
  pub(crate) data: Vector<T>,
  pub(crate) offsets: Vector<usize>,
}

impl<T> CompressedRows<T> {
  /// Empty instance.
  pub const fn new() -> Self {
    Self { data: Vector::new(), offsets: Vector::new() }
  }

  /// Row iterator
  pub fn iter_rows(&self) -> impl Iterator<Item = &[T]> {
    let Self { data, offsets, .. } = self;
    offsets.iter().scan(0usize, |idx, offset| {
      let begin = *idx;
      let end = idx.wrapping_add(*offset);
      *idx = end;
      data.get(begin..end)
    })
  }

  /// Appends a new row from an copyable element.
  pub fn push_row_from_copyable_slice(&mut self, slice: &[T]) -> crate::Result<()>
  where
    T: Copy,
  {
    let Self { data, offsets } = self;
    data.extend_from_copyable_slice(slice)?;
    offsets.push(slice.len())?;
    Ok(())
  }

  /// The number of rows
  pub fn rows(&self) -> usize {
    self.offsets.len()
  }
}
