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
#[allow(non_camel_case_types, reason = "Useful for readability")]
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
  #[cfg(feature = "embassy-net")]
  EmbassyNet(embassy_net::tcp::Error),
  #[cfg(feature = "base64")]
  EncodeSliceError(base64::EncodeSliceError),
  #[cfg(feature = "flate2")]
  Flate2CompressError(flate2::CompressError),
  #[cfg(feature = "flate2")]
  Flate2DecompressError(Box<flate2::DecompressError>),
  #[cfg(feature = "glommio")]
  Glommio(Box<glommio::GlommioError<()>>),
  #[cfg(feature = "httparse")]
  HttpParse(httparse::Error),
  #[cfg(feature = "digest")]
  MacError(digest::MacError),
  #[cfg(feature = "miniserde")]
  Miniserde(miniserde::Error),
  #[cfg(feature = "postgres")]
  PostgresDbError(Box<crate::database::client::postgres::DbError>),
  #[cfg(feature = "protobuf")]
  Protobuf(protobuf::Error),
  #[cfg(feature = "rkyv")]
  RkyvDer(&'static str),
  #[cfg(feature = "rkyv")]
  RkyvSer(Box<RkyvSer>),
  #[cfg(feature = "serde_json")]
  SerdeJson(serde_json::Error),
  #[cfg(feature = "serde-xml-rs")]
  SerdeXmlRs(Box<serde_xml_rs::Error>),
  #[cfg(feature = "serde_yaml")]
  SerdeYaml(serde_yaml::Error),
  #[cfg(feature = "simd-json")]
  SimdJson(Box<simd_json::Error>),
  #[cfg(feature = "smoltcp")]
  SmoltcpTcpRecvError(smoltcp::socket::tcp::RecvError),
  #[cfg(feature = "smoltcp")]
  SmoltcpTcpSendError(smoltcp::socket::tcp::SendError),
  #[cfg(feature = "embedded-tls")]
  TlsError(embedded_tls::TlsError),
  #[cfg(feature = "tokio")]
  TokioJoinError(Box<tokio::task::JoinError>),
  #[cfg(feature = "tokio-rustls")]
  TokioRustlsError(Box<tokio_rustls::rustls::Error>),
  #[cfg(feature = "_tracing-subscriber")]
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

  // Internal
  //
  ArrayStringError(ArrayStringError),
  ArrayVectorError(ArrayVectorError),
  BlocksQueueError(BlocksQueueError),
  #[cfg(feature = "client-api-framework")]
  ClientApiFrameworkError(crate::client_api_framework::ClientApiFrameworkError),
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

  /// A "null" field received from the database was decoded as a non-nullable type or value.
  DB_MissingFieldDataInDecoding,

  /// `GenericTime` needs a backend
  MISC_GenericTimeNeedsBackend,
  /// The hardware returned an incorrect time value
  MISC_InvalidHardwareTime,
  /// Invalid UTF-8.
  MISC_InvalidUTF8,
  /// Indices are out-of-bounds or the number of bytes are too small.
  MISC_InvalidPartitionedBufferBounds,
  /// An expected value could not be found
  MISC_InvalidDatabaseUrl(&'static str),
  /// Backend couldn't perform passed query string
  MISC_InvalidSqlQuery,
  /// Invalid URL
  MISC_InvalidUrl,
  /// Environment variable is not present
  MISC_MissingEnvVar,
  /// A variant used to transform `Option`s into `Result`s
  MISC_NoInnerValue(&'static str),
  /// A set of arithmetic operations resulted in an overflow, underflow or division by zero
  MISC_OutOfBoundsArithmetic,
  /// A buffer was partially read or write but should in fact be fully processed.
  MISC_UnexpectedBufferState,
  /// Unexpected end of file when reading from a stream.
  MISC_UnexpectedStreamEOF,
  /// Unexpected String
  MISC_UnexpectedString {
    length: usize,
  },
  /// Unexpected Unsigned integer
  MISC_UnexpectedUint {
    received: u32,
  },
  /// Unexpected Unsigned integer
  MISC_UnboundedNumber {
    expected: RangeInclusive<u32>,
    received: u32,
  },
  /// Only appending is possible but overwritten is still viable through resetting.
  MISC_UriCanNotBeOverwritten,
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

#[cfg(feature = "embassy-net")]
impl From<embassy_net::tcp::Error> for Error {
  #[inline]
  fn from(from: embassy_net::tcp::Error) -> Self {
    Self::EmbassyNet(from)
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

#[cfg(feature = "miniserde")]
impl From<miniserde::Error> for Error {
  #[inline]
  fn from(from: miniserde::Error) -> Self {
    Self::Miniserde(from)
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

#[cfg(feature = "protobuf")]
impl From<protobuf::Error> for Error {
  #[inline]
  fn from(from: protobuf::Error) -> Self {
    Self::Protobuf(from)
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

#[cfg(feature = "serde-xml-rs")]
impl From<serde_xml_rs::Error> for Error {
  #[inline]
  fn from(from: serde_xml_rs::Error) -> Self {
    Self::SerdeXmlRs(from.into())
  }
}

#[cfg(feature = "serde_yaml")]
impl From<serde_yaml::Error> for Error {
  #[inline]
  fn from(from: serde_yaml::Error) -> Self {
    Self::SerdeYaml(from)
  }
}

#[cfg(feature = "simd-json")]
impl From<simd_json::Error> for Error {
  #[inline]
  fn from(from: simd_json::Error) -> Self {
    Self::SimdJson(from.into())
  }
}

#[cfg(feature = "smoltcp")]
impl From<smoltcp::socket::tcp::RecvError> for Error {
  #[inline]
  fn from(from: smoltcp::socket::tcp::RecvError) -> Self {
    Self::SmoltcpTcpRecvError(from)
  }
}

#[cfg(feature = "smoltcp")]
impl From<smoltcp::socket::tcp::SendError> for Error {
  #[inline]
  fn from(from: smoltcp::socket::tcp::SendError) -> Self {
    Self::SmoltcpTcpSendError(from)
  }
}

#[cfg(feature = "embedded-tls")]
impl From<embedded_tls::TlsError> for Error {
  #[inline]
  fn from(from: embedded_tls::TlsError) -> Self {
    Self::TlsError(from)
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

#[cfg(feature = "_tracing-subscriber")]
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
