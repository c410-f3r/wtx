use crate::ExpectedHeader;
use core::{
  fmt::{Debug, Display, Formatter},
  num::TryFromIntError,
};

/// Grouped individual errors
//
// * `Invalid` Something is present but has invalid state.
// * `Missing`: Not present when expected to be.
// * `Unexpected`: Received something that was not intended.
#[derive(Debug)]
pub enum Error {
  /// Invalid UTF-8.
  InvalidUTF8,
  /// Indices are out-of-bounds or the number of bytes are too small.
  InvalidPartitionedBufferBounds,

  /// Missing Header
  MissingHeader {
    /// See [ExpectedHeader].
    expected: ExpectedHeader,
  },
  /// Url does not contain a host.
  MissingHost,
  /// A value from an expected `key=value` structure was not found
  MissingValue,

  /// A buffer was partially read or write but should in fact be fully processed.
  UnexpectedBufferState,
  /// HTTP version does not match the expected method.
  UnexpectedHttpMethod,
  /// HTTP version does not match the expected value.
  UnexpectedHttpVersion,
  /// Unexpected end of file when reading.
  UnexpectedEOF,

  /// HTTP headers must be unique.
  DuplicatedHeader,
  /// The system does not process HTTP messages greater than 2048 bytes.
  VeryLargeHttp,

  // External
  //
  #[cfg(feature = "deadpool")]
  /// See [deadpool::managed::PoolError].
  DeadPoolManagedPoolError(deadpool::managed::PoolError<()>),
  #[cfg(feature = "deadpool")]
  /// See [deadpool::unmanaged::PoolError].
  DeadPoolUnmanagedPoolError(deadpool::unmanaged::PoolError),
  #[cfg(feature = "flate2")]
  /// See [flate2::CompressError].
  Flate2CompressError(flate2::CompressError),
  #[cfg(feature = "flate2")]
  /// See [flate2::DecompressError].
  Flate2DecompressError(Box<flate2::DecompressError>),
  /// See [glommio::GlommioError].
  #[cfg(feature = "glommio")]
  Glommio(Box<glommio::GlommioError<()>>),
  #[cfg(feature = "httparse")]
  /// See [httparse::Error].
  HttpParse(httparse::Error),
  #[cfg(feature = "std")]
  /// See [std::io::Error]
  IoError(std::io::Error),
  /// See [core::num::ParseIntError].
  ParseIntError(core::num::ParseIntError),
  #[cfg(feature = "tokio-rustls")]
  /// See [tokio_rustls::rustls::Error].
  TokioRustLsError(Box<tokio_rustls::rustls::Error>),
  /// See [TryFromIntError]
  TryFromIntError(TryFromIntError),
  /// See [crate::web_socket::WebSocketError].
  WebSocketError(crate::web_socket::WebSocketError),
}

impl Display for Error {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    <Self as Debug>::fmt(self, f)
  }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

#[cfg(feature = "deadpool")]
impl<T> From<deadpool::managed::PoolError<T>> for Error {
  #[inline]
  fn from(from: deadpool::managed::PoolError<T>) -> Self {
    use deadpool::managed::{HookError, PoolError};
    Self::DeadPoolManagedPoolError(match from {
      PoolError::Timeout(elem) => PoolError::Timeout(elem),
      PoolError::Backend(_) => PoolError::Backend(()),
      PoolError::Closed => PoolError::Closed,
      PoolError::NoRuntimeSpecified => PoolError::NoRuntimeSpecified,
      PoolError::PostCreateHook(elem) => PoolError::PostCreateHook(match elem {
        HookError::Message(elem) => HookError::Message(elem),
        HookError::StaticMessage(elem) => HookError::StaticMessage(elem),
        HookError::Backend(_) => HookError::Backend(()),
      }),
    })
  }
}

#[cfg(feature = "deadpool")]
impl From<deadpool::unmanaged::PoolError> for Error {
  #[inline]
  fn from(from: deadpool::unmanaged::PoolError) -> Self {
    Self::DeadPoolUnmanagedPoolError(from)
  }
}

#[cfg(feature = "flate2")]
impl From<flate2::CompressError> for Error {
  #[inline]
  fn from(from: flate2::CompressError) -> Self {
    Self::Flate2CompressError(from)
  }
}

#[cfg(feature = "flate2")]
impl From<flate2::DecompressError> for Error {
  #[inline]
  fn from(from: flate2::DecompressError) -> Self {
    Self::Flate2DecompressError(from.into())
  }
}

#[cfg(feature = "glommio")]
impl From<glommio::GlommioError<()>> for Error {
  #[inline]
  fn from(from: glommio::GlommioError<()>) -> Self {
    Self::Glommio(from.into())
  }
}

#[cfg(feature = "httparse")]
impl From<httparse::Error> for Error {
  #[inline]
  fn from(from: httparse::Error) -> Self {
    Self::HttpParse(from)
  }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
  #[inline]
  fn from(from: std::io::Error) -> Self {
    Self::IoError(from)
  }
}

#[cfg(feature = "std")]
impl From<core::num::ParseIntError> for Error {
  #[inline]
  fn from(from: core::num::ParseIntError) -> Self {
    Self::ParseIntError(from)
  }
}

impl From<core::str::Utf8Error> for Error {
  #[inline]
  fn from(_: core::str::Utf8Error) -> Self {
    Self::InvalidUTF8
  }
}

#[cfg(feature = "tokio-rustls")]
impl From<tokio_rustls::rustls::Error> for Error {
  #[inline]
  fn from(from: tokio_rustls::rustls::Error) -> Self {
    Self::TokioRustLsError(from.into())
  }
}

impl From<TryFromIntError> for Error {
  #[inline]
  fn from(from: TryFromIntError) -> Self {
    Self::TryFromIntError(from)
  }
}

impl From<crate::web_socket::WebSocketError> for Error {
  #[inline]
  fn from(from: crate::web_socket::WebSocketError) -> Self {
    Self::WebSocketError(from)
  }
}
