use crate::misc::Lease;
use core::fmt::{Display, Write};

/// Query parameters need special handling because of the initial `?`.
#[derive(Debug)]
pub struct QueryWriter<'str, S> {
  initial_len: usize,
  s: &'str mut S,
}

impl<'str, S> QueryWriter<'str, S>
where
  S: Lease<str> + Write,
{
  pub(crate) fn new(s: &'str mut S) -> Self {
    Self { initial_len: s.lease().len(), s }
  }

  /// Writes `?param=value` or `&param=value`.
  #[inline]
  pub fn write<T>(self, param: &str, value: T) -> crate::Result<Self>
  where
    T: Display,
  {
    if self.s.lease().len() == self.initial_len {
      self.s.write_fmt(format_args!("?{param}={value}"))?;
    } else {
      self.s.write_fmt(format_args!("&{param}={value}"))?;
    }
    Ok(self)
  }

  /// Same as [write] but for optional fields.
  #[inline]
  pub fn write_opt<T, U>(self, param: &str, opt: U) -> crate::Result<Self>
  where
    T: Display,
    U: Lease<Option<T>>,
  {
    if let Some(value) = opt.lease() {
      self.write(param, value)
    } else {
      Ok(self)
    }
  }
}
