/// Converts sequences of bytes into numbers.
pub trait FromRadix10: Sized {
  /// Internally uses `atoi` if the feature is active.
  fn from_radix_10(bytes: &[u8]) -> crate::Result<Self>;
}

#[cfg(feature = "atoi")]
impl<T> FromRadix10 for T
where
  T: atoi::FromRadix10SignedChecked,
{
  #[inline]
  fn from_radix_10(bytes: &[u8]) -> crate::Result<Self> {
    atoi::atoi(bytes).ok_or(crate::Error::AtoiInvalidBytes)
  }
}

#[cfg(not(feature = "atoi"))]
impl<T> FromRadix10 for T
where
  T: core::str::FromStr,
  T::Err: Into<crate::Error>,
{
  #[inline]
  fn from_radix_10(bytes: &[u8]) -> crate::Result<Self> {
    crate::misc::from_utf8_basic(bytes)?.parse().map_err(Into::into)
  }
}
