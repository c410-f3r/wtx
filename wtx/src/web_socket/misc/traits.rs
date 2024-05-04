use alloc::vec::Vec;

/// Internal trait not intended for public usage
pub trait Expand {
  /// Internal method not intended for public usage
  fn expand(&mut self, len: usize);
}

impl<T> Expand for &mut T
where
  T: Expand,
{
  fn expand(&mut self, len: usize) {
    (*self).expand(len);
  }
}

impl<T> Expand for Vec<T>
where
  T: Clone + Default,
{
  fn expand(&mut self, len: usize) {
    if len > self.len() {
      self.resize(len, T::default());
    }
  }
}

impl<T> Expand for &mut [T] {
  fn expand(&mut self, _: usize) {}
}

impl<T, const N: usize> Expand for [T; N] {
  fn expand(&mut self, _: usize) {}
}
