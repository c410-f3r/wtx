use crate::collection::Vector;
use alloc::vec::Vec;

/// A trait for extending collections with fallible operations.
pub trait TryExtend<S>
where
  S: ?Sized,
{
  /// Custom error
  type Error;

  /// Attempts to extend this instance with elements from the given `set` source.
  fn try_extend(&mut self, set: &mut S) -> Result<(), Self::Error>;
}

impl<T> TryExtend<[T]> for Vec<T>
where
  T: Copy,
{
  type Error = crate::Error;

  #[inline]
  fn try_extend(&mut self, set: &mut [T]) -> Result<(), Self::Error> {
    self.copy_from_slice(set);
    Ok(())
  }
}

impl<I, T> TryExtend<I> for Vec<T>
where
  I: Iterator<Item = T>,
{
  type Error = crate::Error;

  #[inline]
  fn try_extend(&mut self, set: &mut I) -> Result<(), Self::Error> {
    self.extend(set);
    Ok(())
  }
}

impl<T> TryExtend<[T]> for Vector<T>
where
  T: Copy,
{
  type Error = crate::Error;

  #[inline]
  fn try_extend(&mut self, set: &mut [T]) -> Result<(), Self::Error> {
    self.extend_from_copyable_slice(set)?;
    Ok(())
  }
}

impl<I, T> TryExtend<I> for Vector<T>
where
  I: Iterator<Item = T>,
{
  type Error = crate::Error;

  #[inline]
  fn try_extend(&mut self, set: &mut I) -> Result<(), Self::Error> {
    self.extend_from_iter(set)?;
    Ok(())
  }
}
