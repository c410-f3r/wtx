use crate::{
  collection::Vector,
  misc::{Lease, LeaseMut},
};

/// Helper that appends data into a [`crate::misc::FilledBufferVectorMut`].
#[cfg(any(feature = "http2", feature = "mysql", feature = "postgres", feature = "web-socket"))]
pub type SuffixWriterFbvm<'fb> = SuffixWriter<crate::misc::FilledBufferVectorMut<'fb>>;
/// Helper that appends data into a mutable vector.
pub type SuffixWriterMut<'vec> = SuffixWriter<&'vec mut Vector<u8>>;

/// Helper that appends data
#[derive(Debug)]
pub struct SuffixWriter<V>
where
  V: LeaseMut<Vector<u8>>,
{
  curr_idx: usize,
  initial_idx: usize,
  vec: V,
}

impl<V> SuffixWriter<V>
where
  V: LeaseMut<Vector<u8>>,
{
  /// Iterates over the slice `other`, copies each element, and then appends
  /// it to this vector. The `other` slice is traversed in-order.
  #[inline]
  pub fn extend_from_slice(&mut self, other: &[u8]) -> crate::Result<()> {
    self.extend_from_slices([other])
  }

  #[cfg(any(feature = "postgres", feature = "web-socket-handshake"))]
  pub(crate) fn curr_bytes(&self) -> &[u8] {
    self.vec.lease().get(self.initial_idx..self.curr_idx).unwrap_or_default()
  }

  pub(crate) fn extend_from_slices<'iter, I>(&mut self, slices: I) -> crate::Result<()>
  where
    I: IntoIterator<Item = &'iter [u8]>,
    I::IntoIter: Clone,
  {
    let sum = self.vec.lease_mut().extend_from_copyable_slices(slices)?;
    self.curr_idx = self.curr_idx.wrapping_add(sum);
    Ok(())
  }

  /// The `rn` suffix means that `slice` is copied with a final `\r\n` new line.
  #[cfg(feature = "web-socket-handshake")]
  pub(crate) fn extend_from_slice_rn(&mut self, slice: &[u8]) -> crate::Result<()> {
    self.extend_from_slices([slice, "\r\n".as_bytes()])
  }

  /// The `group_rn` suffix means that only the last slice is copied with a final `\r\n` new line.
  #[cfg(feature = "web-socket-handshake")]
  pub(crate) fn extend_from_slices_group_rn(&mut self, slices: &[&[u8]]) -> crate::Result<()> {
    self.extend_from_slices(slices.iter().copied().chain(["\r\n".as_bytes()]))
  }
}

#[cfg(feature = "postgres")]
impl<V> SuffixWriter<V>
where
  V: LeaseMut<Vector<u8>>,
{
  pub(crate) fn create_buffer(
    &mut self,
    n: usize,
    cb: impl FnOnce(&mut [u8]) -> crate::Result<usize>,
  ) -> crate::Result<()> {
    let prev_len = self.vec.lease().len();
    self.vec.lease_mut().expand(crate::collection::ExpansionTy::Additional(n), 0)?;
    let written = cb(self.vec.lease_mut().get_mut(self.curr_idx..).unwrap_or_default())?;
    self.curr_idx = self.curr_idx.wrapping_add(written);
    self.vec.lease_mut().truncate(prev_len.wrapping_add(written));
    Ok(())
  }

  pub(crate) fn curr_bytes_mut(&mut self) -> &mut [u8] {
    self.vec.lease_mut().get_mut(self.initial_idx..self.curr_idx).unwrap_or_default()
  }

  pub(crate) fn len(&self) -> usize {
    self.curr_idx.wrapping_sub(self.initial_idx)
  }

  pub(crate) fn extend_from_byte(&mut self, byte: u8) -> crate::Result<()> {
    self.extend_from_slices([&[byte][..]])
  }

  /// The `c` suffix means that `slice` is copied as a C string.
  pub(crate) fn extend_from_slice_c(&mut self, slice: &[u8]) -> crate::Result<()> {
    self.extend_from_slices([slice, &[0]])
  }

  /// The `each_c` suffix means that each slice is copied as a C string.
  pub(crate) fn extend_from_slices_each_c(&mut self, slices: &[&[u8]]) -> crate::Result<()> {
    self.extend_from_slices(slices.iter().flat_map(|el| [*el, &[0]]))
  }
}

#[cfg(any(feature = "postgres", feature = "web-socket-handshake"))]
impl<V> SuffixWriter<V>
where
  V: LeaseMut<Vector<u8>>,
{
  pub(crate) fn new(start: usize, vec: V) -> Self {
    Self { curr_idx: start, initial_idx: start, vec }
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
    self.vec.lease()
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
    self.vec.lease_mut().truncate(self.initial_idx);
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
    self.vec.lease_mut().flush()
  }
}
