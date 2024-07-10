use crate::database::orm::SqlValue;
use core::{fmt::Display, hash::Hash};

/// Groups all types that can be a primary key.
pub trait IdValue<E>: Copy + Display + Hash + SqlValue<E> {
  /// Number value using the largest integer representation.
  fn generic(&self) -> u64;
}

impl<E, T> IdValue<E> for &T
where
  E: From<crate::Error>,
  T: IdValue<E>,
{
  #[inline]
  fn generic(&self) -> u64 {
    (**self).generic()
  }
}

impl<E> IdValue<E> for u8
where
  E: From<crate::Error>,
{
  #[inline]
  fn generic(&self) -> u64 {
    (*self).into()
  }
}

impl<E> IdValue<E> for u16
where
  E: From<crate::Error>,
{
  #[inline]
  fn generic(&self) -> u64 {
    (*self).into()
  }
}

impl<E> IdValue<E> for u32
where
  E: From<crate::Error>,
{
  #[inline]
  fn generic(&self) -> u64 {
    (*self).into()
  }
}

impl<E> IdValue<E> for u64
where
  E: From<crate::Error>,
{
  #[inline]
  fn generic(&self) -> u64 {
    *self
  }
}
