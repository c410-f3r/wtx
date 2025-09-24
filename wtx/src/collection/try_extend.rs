use crate::{
  collection::{ArrayString, ArrayVector, LinearStorageLen, Vector},
  misc::{Wrapper, from_utf8_basic},
};
use alloc::vec::Vec;

/// A trait for extending collections with fallible operations.
pub trait TryExtend<S> {
  /// Custom error
  type Error;

  /// Attempts to extend this instance with elements from the given `set` source.
  fn try_extend(&mut self, set: S) -> Result<(), Self::Error>;
}

impl<S, T> TryExtend<S> for &mut T
where
  T: TryExtend<S>,
{
  type Error = T::Error;

  #[inline]
  fn try_extend(&mut self, set: S) -> Result<(), Self::Error> {
    (**self).try_extend(set)
  }
}

impl<'slice, L, const N: usize> TryExtend<&'slice str> for ArrayString<L, N>
where
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_extend(&mut self, set: &'slice str) -> Result<(), Self::Error> {
    self.push_str(set)?;
    Ok(())
  }
}

impl<'slice, L, const N: usize> TryExtend<&'slice [u8]> for ArrayString<L, N>
where
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_extend(&mut self, set: &'slice [u8]) -> Result<(), Self::Error> {
    self.push_str(from_utf8_basic(set)?)?;
    Ok(())
  }
}

impl<L, const M: usize, const N: usize> TryExtend<[u8; M]> for ArrayString<L, N>
where
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_extend(&mut self, set: [u8; M]) -> Result<(), Self::Error> {
    self.push_str(from_utf8_basic(&set)?)?;
    Ok(())
  }
}

impl<'slice, I, L, const N: usize> TryExtend<Wrapper<I>> for ArrayString<L, N>
where
  I: IntoIterator<Item = char>,
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_extend(&mut self, set: Wrapper<I>) -> Result<(), Self::Error> {
    self.extend_from_iter(set.0)?;
    Ok(())
  }
}

impl<'slice, L, T, const N: usize> TryExtend<&'slice [T]> for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
  T: Copy,
{
  type Error = crate::Error;

  #[inline]
  fn try_extend(&mut self, set: &'slice [T]) -> Result<(), Self::Error> {
    self.extend_from_copyable_slice(set)?;
    Ok(())
  }
}

impl<L, T, const M: usize, const N: usize> TryExtend<[T; M]> for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_extend(&mut self, set: [T; M]) -> Result<(), Self::Error> {
    self.extend_from_iter(set)?;
    Ok(())
  }
}

impl<I, L, T, const N: usize> TryExtend<Wrapper<I>> for ArrayVector<L, T, N>
where
  I: IntoIterator<Item = T>,
  L: LinearStorageLen,
{
  type Error = crate::Error;

  #[inline]
  fn try_extend(&mut self, set: Wrapper<I>) -> Result<(), Self::Error> {
    self.extend_from_iter(set.0)?;
    Ok(())
  }
}

impl<'slice, T> TryExtend<&'slice [T]> for Vec<T>
where
  T: Copy,
{
  type Error = crate::Error;

  #[inline]
  fn try_extend(&mut self, set: &'slice [T]) -> Result<(), Self::Error> {
    self.copy_from_slice(set);
    Ok(())
  }
}

impl<T, const N: usize> TryExtend<[T; N]> for Vec<T> {
  type Error = crate::Error;

  #[inline]
  fn try_extend(&mut self, set: [T; N]) -> Result<(), Self::Error> {
    self.extend(set);
    Ok(())
  }
}

impl<I, T> TryExtend<Wrapper<I>> for Vec<T>
where
  I: IntoIterator<Item = T>,
{
  type Error = crate::Error;

  #[inline]
  fn try_extend(&mut self, set: Wrapper<I>) -> Result<(), Self::Error> {
    self.extend(set.0);
    Ok(())
  }
}

impl<'slice, T> TryExtend<&'slice [T]> for Vector<T>
where
  T: Copy,
{
  type Error = crate::Error;

  #[inline]
  fn try_extend(&mut self, set: &'slice [T]) -> Result<(), Self::Error> {
    self.extend_from_copyable_slice(set)?;
    Ok(())
  }
}

impl<T, const N: usize> TryExtend<[T; N]> for Vector<T> {
  type Error = crate::Error;

  #[inline]
  fn try_extend(&mut self, set: [T; N]) -> Result<(), Self::Error> {
    self.extend_from_iter(set)?;
    Ok(())
  }
}

impl<I, T> TryExtend<Wrapper<I>> for Vector<T>
where
  I: IntoIterator<Item = T>,
{
  type Error = crate::Error;

  #[inline]
  fn try_extend(&mut self, set: Wrapper<I>) -> Result<(), Self::Error> {
    self.extend_from_iter(set.0)?;
    Ok(())
  }
}
