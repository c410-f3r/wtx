use alloc::vec::Vec;

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
