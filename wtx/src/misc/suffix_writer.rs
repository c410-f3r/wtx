use crate::{
  collection::{ExpansionTy, Vector},
  misc::{FilledBufferVectorMut, Lease, LeaseMut},
};

/// Helper that appends data into a [`FilledBufferVectorMut`].
pub type SuffixWriterFbvm<'fb> = SuffixWriter<FilledBufferVectorMut<'fb>>;
/// Helper that appends data into a mutable vector.
pub type SuffixWriterMut<'vec> = SuffixWriter<&'vec mut Vector<u8>>;

/// Helper that appends data
#[derive(Debug)]
pub struct SuffixWriter<V>
where
  V: LeaseMut<Vector<u8>>,
{
  _curr_idx: usize,
  _initial_idx: usize,
  _vec: V,
}

impl<V> SuffixWriter<V>
where
  V: LeaseMut<Vector<u8>>,
{
  #[inline]
  pub(crate) fn _new(start: usize, vec: V) -> Self {
    Self { _curr_idx: start, _initial_idx: start, _vec: vec }
  }

  /// Iterates over the slice `other`, copies each element, and then appends
  /// it to this vector. The `other` slice is traversed in-order.
  #[inline]
  pub fn extend_from_slice(&mut self, other: &[u8]) -> crate::Result<()> {
    self._extend_from_slices([other])
  }

  #[inline]
  pub(crate) fn _create_buffer(
    &mut self,
    n: usize,
    cb: impl FnOnce(&mut [u8]) -> crate::Result<usize>,
  ) -> crate::Result<()> {
    let prev_len = self._vec.lease().len();
    self._vec.lease_mut().expand(ExpansionTy::Additional(n), 0)?;
    let written = cb(self._vec.lease_mut().get_mut(self._curr_idx..).unwrap_or_default())?;
    self._curr_idx = self._curr_idx.wrapping_add(written);
    self._vec.lease_mut().truncate(prev_len.wrapping_add(written));
    Ok(())
  }

  #[inline]
  pub(crate) fn _curr_bytes(&self) -> &[u8] {
    self._vec.lease().get(self._initial_idx..self._curr_idx).unwrap_or_default()
  }

  #[inline]
  pub(crate) fn _curr_bytes_mut(&mut self) -> &mut [u8] {
    self._vec.lease_mut().get_mut(self._initial_idx..self._curr_idx).unwrap_or_default()
  }

  #[inline]
  pub(crate) fn _len(&self) -> usize {
    self._curr_idx.wrapping_sub(self._initial_idx)
  }

  #[inline]
  pub(crate) fn _extend_from_byte(&mut self, byte: u8) -> crate::Result<()> {
    self._extend_from_slices([&[byte][..]])
  }

  #[inline]
  pub(crate) fn _extend_from_slices<'iter, I>(&mut self, slices: I) -> crate::Result<()>
  where
    I: IntoIterator<Item = &'iter [u8]>,
    I::IntoIter: Clone,
  {
    let sum = self._vec.lease_mut().extend_from_copyable_slices(slices)?;
    self._curr_idx = self._curr_idx.wrapping_add(sum);
    Ok(())
  }

  /// The `c` suffix means that `slice` is copied as a C string.
  #[inline]
  pub(crate) fn _extend_from_slice_c(&mut self, slice: &[u8]) -> crate::Result<()> {
    self._extend_from_slices([slice, &[0]])
  }

  /// The `each_c` suffix means that each slice is copied as a C string.
  #[inline]
  pub(crate) fn _extend_from_slices_each_c(&mut self, slices: &[&[u8]]) -> crate::Result<()> {
    self._extend_from_slices(slices.iter().flat_map(|el| [*el, &[0]]))
  }

  /// The `rn` suffix means that `slice` is copied with a final `\r\n` new line.
  #[inline]
  pub(crate) fn _extend_from_slice_rn(&mut self, slice: &[u8]) -> crate::Result<()> {
    self._extend_from_slices([slice, "\r\n".as_bytes()])
  }

  /// The `group_rn` suffix means that only the last slice is copied with a final `\r\n` new line.
  #[inline]
  pub(crate) fn _extend_from_slices_group_rn(&mut self, slices: &[&[u8]]) -> crate::Result<()> {
    self._extend_from_slices(slices.iter().copied().chain(["\r\n".as_bytes()]))
  }
}

impl<V> Lease<SuffixWriter<V>> for SuffixWriter<V>
where
  V: LeaseMut<Vector<u8>>,
{
  #[inline]
  fn lease(&self) -> &SuffixWriter<V> {
    self
  }
}

impl<V> Lease<[u8]> for SuffixWriter<V>
where
  V: LeaseMut<Vector<u8>>,
{
  #[inline]
  fn lease(&self) -> &[u8] {
    self._vec.lease()
  }
}

impl<V> LeaseMut<SuffixWriter<V>> for SuffixWriter<V>
where
  V: LeaseMut<Vector<u8>>,
{
  #[inline]
  fn lease_mut(&mut self) -> &mut SuffixWriter<V> {
    self
  }
}

impl<V> Drop for SuffixWriter<V>
where
  V: LeaseMut<Vector<u8>>,
{
  #[inline]
  fn drop(&mut self) {
    self._vec.lease_mut().truncate(self._initial_idx);
  }
}

#[cfg(feature = "std")]
impl<V> std::io::Write for SuffixWriter<V>
where
  V: LeaseMut<Vector<u8>>,
{
  #[inline]
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    self.extend_from_slice(buf).map_err(std::io::Error::other)?;
    Ok(buf.len())
  }

  #[inline]
  fn flush(&mut self) -> std::io::Result<()> {
    self._vec.lease_mut().flush()
  }
}
