use crate::{
  collections::{Truncate, TryExtend, Vector},
  misc::{Lease, LeaseMut, SingleTypeStorage, Wrapper},
};
use core::{iter, ops::Range};

/// [`SuffixPusher`] with a mutable vector reference.
pub type SuffixPusherVectorMut<'inner, T> = SuffixPusher<&'inner mut Vector<T>>;

/// A scoped writer that appends a temporary suffix to a collection.
#[derive(Debug)]
pub struct SuffixPusher<T>
where
  T: Truncate<usize>,
{
  initial_idx: usize,
  inner: T,
}

impl<T> SuffixPusher<T>
where
  T: Truncate<usize>,
{
  /// Inner collection
  #[inline]
  pub const fn inner(&self) -> &T {
    &self.inner
  }

  /// Inner mutable collection
  #[inline]
  pub const fn inner_mut(&mut self) -> &mut T {
    &mut self.inner
  }
}

impl<T, U> SuffixPusher<T>
where
  T: Lease<[U]> + SingleTypeStorage<Item = U> + Truncate<usize>,
{
  /// All bytes written after the creation of this instance
  #[inline]
  pub fn curr(&self) -> &[U] {
    self.inner.lease().get(self.initial_idx..).unwrap_or_default()
  }
}

impl<T, U> SuffixPusher<T>
where
  T: LeaseMut<[U]> + SingleTypeStorage<Item = U> + Truncate<usize>,
{
  /// All mutable bytes written after the creation of this instance
  #[inline]
  pub fn curr_mut(&mut self) -> &mut [U] {
    self.inner.lease_mut().get_mut(self.initial_idx..).unwrap_or_default()
  }
}

impl<T, U> SuffixPusher<T>
where
  T: LeaseMut<[U]>
    + SingleTypeStorage<Item = U>
    + Truncate<usize>
    + TryExtend<Wrapper<iter::Map<Range<usize>, fn(usize) -> U>>>,
  U: Default,
{
  /// Reserves space and allows writing to it via a closure.
  #[cfg(feature = "postgres")]
  #[inline]
  pub(crate) fn reserve_and_write(
    &mut self,
    additional_bytes: usize,
    cb: impl FnOnce(&mut [U]) -> crate::Result<usize>,
  ) -> crate::Result<()> {
    let prev_len = self.inner.lease().len();
    self.inner.try_extend(Wrapper((0..additional_bytes).map(|_| U::default())))?;
    let written = cb(self.inner.lease_mut().get_mut(prev_len..).unwrap_or_default())?;
    self.inner.truncate(prev_len.wrapping_add(written));
    Ok(())
  }
}

impl<T, U> From<T> for SuffixPusher<T>
where
  T: Lease<[U]> + SingleTypeStorage<Item = U> + Truncate<usize>,
{
  #[inline]
  fn from(value: T) -> Self {
    Self { initial_idx: value.lease().len(), inner: value }
  }
}

impl<T> Lease<SuffixPusher<T>> for SuffixPusher<T>
where
  T: Truncate<usize>,
{
  #[inline]
  fn lease(&self) -> &SuffixPusher<T> {
    self
  }
}

impl<T> LeaseMut<SuffixPusher<T>> for SuffixPusher<T>
where
  T: Truncate<usize>,
{
  #[inline]
  fn lease_mut(&mut self) -> &mut SuffixPusher<T> {
    self
  }
}

impl<T> Drop for SuffixPusher<T>
where
  T: Truncate<usize>,
{
  #[inline]
  fn drop(&mut self) {
    self.inner.truncate(self.initial_idx);
  }
}

impl<T> core::fmt::Write for SuffixPusher<T>
where
  T: Truncate<usize> + for<'any> TryExtend<&'any [u8]>,
{
  #[inline]
  fn write_str(&mut self, s: &str) -> core::fmt::Result {
    self.inner.try_extend(s.as_bytes()).map_err(|_err| core::fmt::Error)?;
    Ok(())
  }
}

#[cfg(feature = "std")]
impl<T> std::io::Write for SuffixPusher<T>
where
  T: Truncate<usize> + std::io::Write,
{
  #[inline]
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    self.inner.write(buf)
  }

  #[inline]
  fn flush(&mut self) -> std::io::Result<()> {
    self.inner.flush()
  }
}
