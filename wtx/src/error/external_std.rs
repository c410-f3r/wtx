#[allow(unused_imports, reason = "Depends on the selection of features")]
use alloc::boxed::Box;
use alloc::string::String;
use core::{any::Any, array::TryFromSliceError, fmt::Write as _, slice::GetDisjointMutError};

use crate::{Error, RecvError, SendError};

impl From<core::net::AddrParseError> for Error {
  #[inline]
  fn from(from: core::net::AddrParseError) -> Self {
    Self::AddrParseError(from)
  }
}

impl From<core::fmt::Error> for Error {
  #[inline]
  fn from(from: core::fmt::Error) -> Self {
    Self::Fmt(from)
  }
}

impl From<GetDisjointMutError> for Error {
  #[inline]
  fn from(from: GetDisjointMutError) -> Self {
    Self::GetDisjointMutError(from)
  }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
  #[inline]
  fn from(from: std::io::Error) -> Self {
    Self::IoError(from)
  }
}

impl From<core::num::ParseIntError> for Error {
  #[inline]
  fn from(from: core::num::ParseIntError) -> Self {
    Self::ParseIntError(from)
  }
}

impl From<RecvError> for Error {
  #[inline]
  fn from(from: RecvError) -> Self {
    Self::RecvError(from)
  }
}

impl From<SendError<()>> for Error {
  #[inline]
  fn from(from: SendError<()>) -> Self {
    Self::SendError(from)
  }
}

impl From<core::num::TryFromIntError> for Error {
  #[inline]
  fn from(from: core::num::TryFromIntError) -> Self {
    Self::TryFromIntError(from)
  }
}

impl From<TryFromSliceError> for Error {
  #[inline]
  fn from(from: TryFromSliceError) -> Self {
    Self::TryFromSliceError(from)
  }
}

impl From<Box<dyn Any + Send + 'static>> for Error {
  #[inline]
  fn from(from: Box<dyn Any + Send + 'static>) -> Self {
    let mut string = String::new();
    string.truncate(1024);
    let _rslt = string.write_fmt(format_args!("{from:?}"));
    Self::Generic(string.try_into().unwrap_or_default())
  }
}
