use crate::misc::{ArrayStringError, ArrayVectorError, BlocksQueueError, QueueError, VectorError};
#[allow(unused_imports, reason = "Depends on the selection of features")]
use alloc::boxed::Box;
use core::{
  fmt::{Debug, Display, Formatter},
  ops::RangeInclusive,
};

#[cfg(target_pointer_width = "64")]
const _: () = {
  assert!(size_of::<Error>() == 24);
};

#[cfg(feature = "rkyv")]
type RkyvSer = rkyv::ser::serializers::CompositeSerializerError<
  core::convert::Infallible,
  rkyv::ser::serializers::AllocScratchError,
  rkyv::ser::serializers::SharedSerializeMapError,
>;

/// Grouped individual errors
#[allow(missing_docs, reason = "Work in progress")]
#[derive(Debug)]
pub enum Error {
  // External - Misc
  //
  AtoiInvalidBytes,
  #[cfg(feature = "chrono")]
  ChronoParseError(chrono::ParseError),
  #[cfg(feature = "cl-aux")]
  ClAux(cl_aux::Error),
  #[cfg(feature = "crypto-common")]
  CryptoCommonInvalidLength(crypto_common::InvalidLength),
  #[cfg(feature = "base64")]
  DecodeError(base64::DecodeError),
  #[cfg(feature = "base64")]
  DecodeSliceError(base64::DecodeSliceError),
  #[cfg(feature = "base64")]
  EncodeSliceError(base64::EncodeSliceError),
  #[cfg(feature = "flate2")]
  Flate2CompressError(flate2::CompressError),
  #[cfg(feature = "flate2")]
  Flate2DecompressError(Box<flate2::DecompressError>),
  #[cfg(feature = "httparse")]
  HttpParse(httparse::Error),
  #[cfg(feature = "digest")]
  MacError(digest::MacError),
  #[cfg(feature = "postgres")]
  PostgresDbError(Box<crate::database::client::postgres::DbError>),
  #[cfg(feature = "quick-protobuf")]
  QuickProtobuf(Box<quick_protobuf::Error>),
  #[cfg(feature = "rkyv")]
  RkyvDer(&'static str),
  #[cfg(feature = "rkyv")]
  RkyvSer(Box<RkyvSer>),
  #[cfg(feature = "serde_json")]
  SerdeJson(serde_json::Error),
  #[cfg(feature = "tokio")]
  TokioJoinError(Box<tokio::task::JoinError>),
  #[cfg(feature = "tokio-rustls")]
  TokioRustlsError(Box<tokio_rustls::rustls::Error>),
  #[cfg(feature = "tracing-subscriber")]
  TryInitError(tracing_subscriber::util::TryInitError),
  #[cfg(feature = "std")]
  TryLockError(std::sync::TryLockError<()>),
  #[cfg(feature = "x509-certificate")]
  X509CertificateError(Box<x509_certificate::X509CertificateError>),

  // External - Std
  //
  AddrParseError(core::net::AddrParseError),
  Fmt(core::fmt::Error),
  #[cfg(feature = "std")]
  IoError(std::io::Error),
  ParseIntError(core::num::ParseIntError),
  TryFromIntError(core::num::TryFromIntError),
  TryFromSliceError(core::array::TryFromSliceError),
  #[cfg(feature = "std")]
  VarError(VarError),
  Utf8Error(core::str::Utf8Error),

  // Generic
  //
  /// `GenericTime` needs a backend
  GenericTimeNeedsBackend,
  /// The hardware returned an incorrect time value
  InvalidHardwareTime,
  /// Indices are out-of-bounds or the number of bytes are too small.
  InvalidPartitionedBufferBounds,
  /// Invalid UTF-8.
  InvalidUTF8,
  /// Invalid URI
  InvalidUri,
  /// There is no CA provider.
  MissingCaProviders,
  /// A variant used to transform `Option`s into `Result`s
  NoInnerValue(&'static str),
  /// A set of arithmetic operations resulted in an overflow, underflow or division by zero
  OutOfBoundsArithmetic,
  /// Unexpected Unsigned integer
  UnboundedNumber {
    expected: RangeInclusive<u32>,
    received: u32,
  },
  /// A buffer was partially read or write but should in fact be fully processed.
  UnexpectedBufferState,
  /// Unexpected end of file when reading from a stream.
  UnexpectedStreamReadEOF,
  /// Unexpected end of file when writing to a stream.
  UnexpectedStreamWriteEOF,
  /// Unexpected String
  UnexpectedString {
    length: usize,
  },
  /// Unexpected Unsigned integer
  UnexpectedUint {
    received: u32,
  },
  /// Only appending is possible but overwritten is still viable through resetting.
  UriCanNotBeOverwritten,

  // Internal
  //
  ArrayStringError(ArrayStringError),
  ArrayVectorError(ArrayVectorError),
  BlocksQueueError(BlocksQueueError),
  #[cfg(feature = "client-api-framework")]
  ClientApiFrameworkError(crate::client_api_framework::ClientApiFrameworkError),
  #[cfg(feature = "database")]
  DatabaseError(crate::database::DatabaseError),
  #[cfg(feature = "data-transformation")]
  DataTransformationError(crate::data_transformation::DataTransformationError),
  #[cfg(feature = "http-client-framework")]
  HttpClientFrameworkError(crate::http::client_framework::HttpClientFrameworkError),
  #[cfg(feature = "http")]
  HttpError(crate::http::HttpError),
  #[cfg(feature = "http2")]
  Http2ErrorGoAway(crate::http2::Http2ErrorCode, Option<crate::http2::Http2Error>),
  #[cfg(feature = "http2")]
  Http2ErrorReset(crate::http2::Http2ErrorCode, Option<crate::http2::Http2Error>, u32),
  #[cfg(feature = "postgres")]
  PostgresError(crate::database::client::postgres::PostgresError),
  QueueError(QueueError),
  #[cfg(feature = "schema-manager")]
  SchemaManagerError(crate::database::schema_manager::SchemaManagerError),
  VectorError(VectorError),
  #[cfg(feature = "web-socket")]
  WebSocketError(crate::web_socket::WebSocketError),
}

impl Display for Error {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    <Self as Debug>::fmt(self, f)
  }
}

impl core::error::Error for Error {}

impl From<Error> for () {
  #[inline]
  fn from(_: Error) -> Self {}
}

#[cfg(feature = "chrono")]
impl From<chrono::ParseError> for Error {
  #[inline]
  #[track_caller]
  fn from(from: chrono::ParseError) -> Self {
    Self::ChronoParseError(from)
  }
}

#[cfg(feature = "cl-aux")]
impl From<cl_aux::Error> for Error {
  #[inline]
  fn from(from: cl_aux::Error) -> Self {
    Self::ClAux(from)
  }
}

#[cfg(feature = "crypto-common")]
impl From<crypto_common::InvalidLength> for Error {
  #[inline]
  #[track_caller]
  fn from(from: crypto_common::InvalidLength) -> Self {
    Self::CryptoCommonInvalidLength(from)
  }
}

#[cfg(feature = "base64")]
impl From<base64::DecodeError> for Error {
  #[inline]
  #[track_caller]
  fn from(from: base64::DecodeError) -> Self {
    Self::DecodeError(from)
  }
}

#[cfg(feature = "base64")]
impl From<base64::DecodeSliceError> for Error {
  #[inline]
  #[track_caller]
  fn from(from: base64::DecodeSliceError) -> Self {
    Self::DecodeSliceError(from)
  }
}

#[cfg(feature = "base64")]
impl From<base64::EncodeSliceError> for Error {
  #[inline]
  fn from(from: base64::EncodeSliceError) -> Self {
    Self::EncodeSliceError(from)
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
impl From<std::env::VarError> for Error {
  #[inline]
  fn from(from: std::env::VarError) -> Self {
    Self::VarError(match from {
      std::env::VarError::NotPresent => VarError::NotPresent,
      std::env::VarError::NotUnicode(_) => VarError::NotUnicode,
    })
  }
}

#[cfg(feature = "digest")]
impl From<digest::MacError> for Error {
  #[inline]
  fn from(from: digest::MacError) -> Self {
    Self::MacError(from)
  }
}

impl From<core::num::ParseIntError> for Error {
  #[inline]
  fn from(from: core::num::ParseIntError) -> Self {
    Self::ParseIntError(from)
  }
}

#[cfg(feature = "postgres")]
impl From<crate::database::client::postgres::DbError> for Error {
  #[inline]
  fn from(from: crate::database::client::postgres::DbError) -> Self {
    Self::PostgresDbError(from.into())
  }
}

#[cfg(feature = "quick-protobuf")]
impl From<quick_protobuf::Error> for Error {
  #[inline]
  fn from(from: quick_protobuf::Error) -> Self {
    Self::QuickProtobuf(from.into())
  }
}

#[cfg(feature = "rkyv")]
impl From<&'static str> for Error {
  #[inline]
  fn from(from: &'static str) -> Self {
    Self::RkyvDer(from)
  }
}

#[cfg(feature = "rkyv")]
impl From<RkyvSer> for Error {
  #[inline]
  fn from(from: RkyvSer) -> Self {
    Self::RkyvSer(from.into())
  }
}

#[cfg(feature = "serde_json")]
impl From<serde_json::Error> for Error {
  #[inline]
  fn from(from: serde_json::Error) -> Self {
    Self::SerdeJson(from)
  }
}

#[cfg(feature = "tokio")]
impl From<tokio::task::JoinError> for Error {
  #[inline]
  fn from(from: tokio::task::JoinError) -> Self {
    Self::TokioJoinError(from.into())
  }
}

#[cfg(feature = "tokio-rustls")]
impl From<tokio_rustls::rustls::Error> for Error {
  #[inline]
  fn from(from: tokio_rustls::rustls::Error) -> Self {
    Self::TokioRustlsError(from.into())
  }
}

#[cfg(feature = "tracing-subscriber")]
impl From<tracing_subscriber::util::TryInitError> for Error {
  #[inline]
  fn from(from: tracing_subscriber::util::TryInitError) -> Self {
    Self::TryInitError(from)
  }
}

impl From<core::num::TryFromIntError> for Error {
  #[inline]
  fn from(from: core::num::TryFromIntError) -> Self {
    Self::TryFromIntError(from)
  }
}

impl From<core::array::TryFromSliceError> for Error {
  #[inline]
  fn from(from: core::array::TryFromSliceError) -> Self {
    Self::TryFromSliceError(from)
  }
}

#[cfg(feature = "std")]
impl<T> From<std::sync::TryLockError<T>> for Error {
  #[inline]
  fn from(from: std::sync::TryLockError<T>) -> Self {
    Self::TryLockError(match from {
      std::sync::TryLockError::Poisoned(_) => {
        std::sync::TryLockError::Poisoned(std::sync::PoisonError::new(()))
      }
      std::sync::TryLockError::WouldBlock => std::sync::TryLockError::WouldBlock,
    })
  }
}

#[cfg(feature = "x509-certificate")]
impl From<x509_certificate::X509CertificateError> for Error {
  #[inline]
  fn from(from: x509_certificate::X509CertificateError) -> Self {
    Self::X509CertificateError(from.into())
  }
}

// Internal

impl From<ArrayStringError> for Error {
  #[inline]
  fn from(from: ArrayStringError) -> Self {
    Self::ArrayStringError(from)
  }
}

impl From<ArrayVectorError> for Error {
  #[inline]
  fn from(from: ArrayVectorError) -> Self {
    Self::ArrayVectorError(from)
  }
}

impl From<BlocksQueueError> for Error {
  #[inline]
  fn from(from: BlocksQueueError) -> Self {
    Self::BlocksQueueError(from)
  }
}

#[cfg(feature = "http")]
impl From<crate::http::HttpError> for Error {
  #[inline]
  fn from(from: crate::http::HttpError) -> Self {
    Self::HttpError(from)
  }
}

#[cfg(feature = "client-api-framework")]
impl From<crate::client_api_framework::ClientApiFrameworkError> for Error {
  #[inline]
  fn from(from: crate::client_api_framework::ClientApiFrameworkError) -> Self {
    Self::ClientApiFrameworkError(from)
  }
}

#[cfg(feature = "database")]
impl From<crate::database::DatabaseError> for Error {
  #[inline]
  fn from(from: crate::database::DatabaseError) -> Self {
    Self::DatabaseError(from)
  }
}

#[cfg(feature = "data-transformation")]
impl From<crate::data_transformation::DataTransformationError> for Error {
  #[inline]
  fn from(from: crate::data_transformation::DataTransformationError) -> Self {
    Self::DataTransformationError(from)
  }
}

#[cfg(feature = "http-client-framework")]
impl From<crate::http::client_framework::HttpClientFrameworkError> for Error {
  #[inline]
  fn from(from: crate::http::client_framework::HttpClientFrameworkError) -> Self {
    Self::HttpClientFrameworkError(from)
  }
}

#[cfg(feature = "postgres")]
impl From<crate::database::client::postgres::PostgresError> for Error {
  #[inline]
  fn from(from: crate::database::client::postgres::PostgresError) -> Self {
    Self::PostgresError(from)
  }
}

impl From<QueueError> for Error {
  #[inline]
  fn from(from: QueueError) -> Self {
    Self::QueueError(from)
  }
}

#[cfg(feature = "schema-manager")]
impl From<crate::database::schema_manager::SchemaManagerError> for Error {
  #[inline]
  fn from(from: crate::database::schema_manager::SchemaManagerError) -> Self {
    Self::SchemaManagerError(from)
  }
}

impl From<VectorError> for Error {
  #[inline]
  fn from(from: VectorError) -> Self {
    Self::VectorError(from)
  }
}

#[cfg(feature = "web-socket")]
impl From<crate::web_socket::WebSocketError> for Error {
  #[inline]
  fn from(from: crate::web_socket::WebSocketError) -> Self {
    Self::WebSocketError(from)
  }
}

/// The error type for operations interacting with environment variables.
#[cfg(feature = "std")]
#[derive(Clone, Copy, Debug)]
pub enum VarError {
  /// The specified environment variable was not present in the current
  /// process's environment.
  NotPresent,

  /// The specified environment variable was found, but it did not contain
  /// valid unicode data. The found data is returned as a payload of this
  /// variant.
  NotUnicode,
}
