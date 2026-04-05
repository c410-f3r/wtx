/// A specified version of `Default` indented only for arrays of arbitrary sizes.
//
// FIXME(stable): impl Default for [T; N]
pub trait DefaultArray {
  /// Returns the "default value" for the array.
  fn default_array() -> Self;
}

impl<T, const N: usize> DefaultArray for [T; N]
where
  T: Default,
{
  #[inline]
  fn default_array() -> Self {
    core::array::from_fn(|_| T::default())
  }
}
