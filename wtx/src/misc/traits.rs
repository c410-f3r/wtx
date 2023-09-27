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
      self.resize(len, <_>::default());
    }
  }
}

impl<T> Expand for &mut [T] {
  fn expand(&mut self, _: usize) {}
}

impl<T, const N: usize> Expand for [T; N] {
  fn expand(&mut self, _: usize) {}
}

/// Internal trait not intended for public usage
pub trait SingleTypeStorage {
  /// Internal method not intended for public usage
  type Item;
}

impl<T> SingleTypeStorage for &T
where
  T: SingleTypeStorage,
{
  type Item = T::Item;
}

impl<T> SingleTypeStorage for &mut T
where
  T: SingleTypeStorage,
{
  type Item = T::Item;
}

impl<T, const N: usize> SingleTypeStorage for [T; N] {
  type Item = T;
}

impl<T> SingleTypeStorage for &'_ [T] {
  type Item = T;
}

impl<T> SingleTypeStorage for &'_ mut [T] {
  type Item = T;
}
impl<T> SingleTypeStorage for Vec<T> {
  type Item = T;
}
