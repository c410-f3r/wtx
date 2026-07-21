use crate::{
  collections::{SingleTypeStorage, Truncate, TryExtend, Vector},
  misc::{Lease, LeaseMut, SensitiveBytes},
};
use core::ops::{Deref, DerefMut};

/// [`SuffixGuard`] with a mutable vector reference.
pub type SuffixGuardVectorMut<'inner, T> = SuffixGuard<&'inner mut Vector<T>>;

#[derive(Debug)]
/// A [`SuffixGuard`] where the elements are bytes that can also be zeroed.
pub struct SensitiveSuffixGuard<T>(SuffixGuard<T>)
where
  T: LeaseMut<[u8]> + SingleTypeStorage<Item = u8> + Truncate<usize>;

impl<T> Deref for SensitiveSuffixGuard<T>
where
  T: LeaseMut<[u8]> + SingleTypeStorage<Item = u8> + Truncate<usize>,
{
  type Target = SuffixGuard<T>;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T> DerefMut for SensitiveSuffixGuard<T>
where
  T: LeaseMut<[u8]> + SingleTypeStorage<Item = u8> + Truncate<usize>,
{
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl<T> Drop for SensitiveSuffixGuard<T>
where
  T: LeaseMut<[u8]> + SingleTypeStorage<Item = u8> + Truncate<usize>,
{
  #[inline]
  fn drop(&mut self) {
    drop(SensitiveBytes::new(self.0.curr_mut()));
  }
}

impl<T> From<T> for SensitiveSuffixGuard<T>
where
  T: LeaseMut<[u8]> + SingleTypeStorage<Item = u8> + Truncate<usize>,
{
  #[inline]
  fn from(value: T) -> Self {
    Self(SuffixGuard::<T>::from(value))
  }
}

/// A scoped writer that appends a temporary suffix to a collection.
#[derive(Debug)]
pub struct SuffixGuard<T>
where
  T: Truncate<usize>,
{
  initial_idx: usize,
  inner: T,
}

impl<T> SuffixGuard<T>
where
  T: Truncate<usize>,
{
  /// Initial buffer index when this instance was created.
  #[inline]
  pub const fn idx(&self) -> usize {
    self.initial_idx
  }

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

  /// Mutable parts
  #[inline]
  pub const fn parts_mut(&mut self) -> (usize, &mut T) {
    (self.initial_idx, &mut self.inner)
  }
}

impl<T, U> SuffixGuard<T>
where
  T: Lease<[U]> + SingleTypeStorage<Item = U> + Truncate<usize>,
{
  /// All bytes written after the creation of this instance
  #[inline]
  pub fn curr(&self) -> &[U] {
    self.inner.lease().get(self.initial_idx..).unwrap_or_default()
  }
}

impl<T, U> SuffixGuard<T>
where
  T: LeaseMut<[U]> + SingleTypeStorage<Item = U> + Truncate<usize>,
{
  /// All mutable bytes written after the creation of this instance
  #[inline]
  pub fn curr_mut(&mut self) -> &mut [U] {
    self.inner.lease_mut().get_mut(self.initial_idx..).unwrap_or_default()
  }
}

impl<T, U> From<T> for SuffixGuard<T>
where
  T: Lease<[U]> + SingleTypeStorage<Item = U> + Truncate<usize>,
{
  #[inline]
  fn from(value: T) -> Self {
    Self { initial_idx: value.lease().len(), inner: value }
  }
}

impl<T> Lease<SuffixGuard<T>> for SuffixGuard<T>
where
  T: Truncate<usize>,
{
  #[inline]
  fn lease(&self) -> &SuffixGuard<T> {
    self
  }
}

impl<T> LeaseMut<SuffixGuard<T>> for SuffixGuard<T>
where
  T: Truncate<usize>,
{
  #[inline]
  fn lease_mut(&mut self) -> &mut SuffixGuard<T> {
    self
  }
}

impl<T> Drop for SuffixGuard<T>
where
  T: Truncate<usize>,
{
  #[inline]
  fn drop(&mut self) {
    self.inner.truncate(self.initial_idx);
  }
}

impl<T> core::fmt::Write for SuffixGuard<T>
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
impl<T> std::io::Write for SuffixGuard<T>
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
