use crate::{collection::Vector, misc::Wrapper};
use alloc::vec::Vec;

/// A trait for extending collections with fallible operations.
pub trait TryExtend<S> {
  /// Custom error
  type Error;

  /// Attempts to extend this instance with elements from the given `set` source.
  fn try_extend(&mut self, set: S) -> Result<(), Self::Error>;
}

impl<'slice, T> TryExtend<&'slice mut [T]> for Vec<T>
where
  T: Copy,
{
  type Error = crate::Error;

  #[inline]
  fn try_extend(&mut self, set: &'slice mut [T]) -> Result<(), Self::Error> {
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

impl<'slice, T> TryExtend<&'slice mut [T]> for Vector<T>
where
  T: Copy,
{
  type Error = crate::Error;

  #[inline]
  fn try_extend(&mut self, set: &'slice mut [T]) -> Result<(), Self::Error> {
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
