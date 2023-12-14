use alloc::string::String;
use core::fmt::Write;

/// Raw SQL representation of a type
pub trait SqlValue<E> {
  /// Pushes the representation into `buffer_cmd`.
  fn write(&self, buffer_cmd: &mut String) -> Result<(), E>;
}

impl<E, T> SqlValue<E> for &'_ T
where
  T: SqlValue<E>,
{
  #[inline]
  fn write(&self, buffer_cmd: &mut String) -> Result<(), E> {
    (**self).write(buffer_cmd)
  }
}

impl<E, T> SqlValue<E> for Option<T>
where
  T: SqlValue<E>,
{
  #[inline]
  fn write(&self, buffer_cmd: &mut String) -> Result<(), E> {
    if let Some(ref elem) = *self {
      elem.write(buffer_cmd)
    } else {
      buffer_cmd.push_str("null");
      Ok(())
    }
  }
}

macro_rules! impl_display {
  ($ty:ty $(, $($bounds:tt)+)?) => {
    impl<E, $($($bounds)+)?> SqlValue<E> for $ty
    where
      E: From<crate::Error>
    {
      #[inline]
      fn write(&self, buffer_cmd: &mut String) -> Result<(), E> {
        buffer_cmd.write_fmt(format_args!("'{self}'")).map_err(From::from)?;
        Ok(())
      }
    }
  }
}

impl_display!(&'_ str);
#[cfg(feature = "arrayvec")]
impl_display!(arrayvec::ArrayString<N>, const N: usize);
impl_display!(bool);
impl_display!(i32);
impl_display!(i64);
impl_display!(u32);
impl_display!(u64);
impl_display!(String);

#[cfg(feature = "rust_decimal")]
impl_display!(rust_decimal::Decimal);
