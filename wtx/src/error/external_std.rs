use crate::{Error, RecvError, SendError, error::oapque_string_err};
#[allow(unused_imports, reason = "Depends on the selection of features")]
use alloc::boxed::Box;
use core::{any::Any, array::TryFromSliceError, slice::GetDisjointMutError};

impl From<core::net::AddrParseError> for Error {
  #[inline]
  fn from(from: core::net::AddrParseError) -> Self {
    Self::AddrParse(from)
  }
}

impl From<core::fmt::Error> for Error {
  #[inline]
  fn from(from: core::fmt::Error) -> Self {
    Self::Fmt(from)
  }
}

impl From<alloc::string::FromUtf8Error> for Error {
  #[inline]
  fn from(from: alloc::string::FromUtf8Error) -> Self {
    Self::FromUtf8(from.into())
  }
}

impl From<GetDisjointMutError> for Error {
  #[inline]
  fn from(from: GetDisjointMutError) -> Self {
    Self::GetDisjointMut(from)
  }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
  #[inline]
  fn from(from: std::io::Error) -> Self {
    Self::Io(from)
  }
}

impl From<core::num::ParseFloatError> for Error {
  #[inline]
  fn from(from: core::num::ParseFloatError) -> Self {
    Self::ParseFloat(from)
  }
}

impl From<core::num::ParseIntError> for Error {
  #[inline]
  fn from(from: core::num::ParseIntError) -> Self {
    Self::ParseInt(from)
  }
}

impl From<RecvError> for Error {
  #[inline]
  fn from(from: RecvError) -> Self {
    Self::Recv(from)
  }
}

impl From<SendError<()>> for Error {
  #[inline]
  fn from(from: SendError<()>) -> Self {
    Self::Send(from)
  }
}

impl From<core::num::TryFromIntError> for Error {
  #[inline]
  fn from(from: core::num::TryFromIntError) -> Self {
    Self::TryFromInt(from)
  }
}

impl From<TryFromSliceError> for Error {
  #[inline]
  fn from(from: TryFromSliceError) -> Self {
    Self::TryFromSlice(from)
  }
}

impl From<core::str::Utf8Error> for Error {
  #[inline]
  fn from(from: core::str::Utf8Error) -> Self {
    Self::Utf8(from.into())
  }
}

impl From<Box<dyn Any + Send + 'static>> for Error {
  #[inline]
  fn from(from: Box<dyn Any + Send + 'static>) -> Self {
    Self::Generic(oapque_string_err(format_args!("{from:?}")).try_into().unwrap_or_default())
  }
}
