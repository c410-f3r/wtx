use crate::misc::{BufferParam, Vector};

/// Internal trait not intended for public usage
pub trait Expand {
  /// Internal method not intended for public usage
  fn expand(&mut self, len: usize) -> crate::Result<()>;
}

impl<T> Expand for &mut T
where
  T: Expand,
{
  #[inline]
  fn expand(&mut self, len: usize) -> crate::Result<()> {
    (*self).expand(len)
  }
}

impl<T> Expand for Vector<T>
where
  T: Clone + Default,
{
  #[inline]
  fn expand(&mut self, len: usize) -> crate::Result<()> {
    self.expand(BufferParam::Len(len), T::default())?;
    Ok(())
  }
}

impl<T> Expand for &mut [T] {
  #[inline]
  fn expand(&mut self, _: usize) -> crate::Result<()> {
    Ok(())
  }
}

impl<T, const N: usize> Expand for [T; N] {
  #[inline]
  fn expand(&mut self, _: usize) -> crate::Result<()> {
    Ok(())
  }
}
