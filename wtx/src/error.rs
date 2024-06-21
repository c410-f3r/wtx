use crate::{
  http::{KnownHeaderName, Method},
  misc::{ArrayStringError, ArrayVectorError, BlocksQueueError, QueueError, VectorError},
};
use core::{
  fmt::{Debug, Display, Formatter},
  ops::RangeInclusive,
};
#[allow(unused_imports)]
use {alloc::boxed::Box, alloc::string::String};

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
#[allow(missing_docs, non_camel_case_types)]
#[derive(Debug)]
pub enum Error {
  // External - Misc
  //
  AtoiInvalidBytes,
  #[cfg(feature = "chrono")]
  ChronoParseError(chrono::ParseError),
  #[cfg(feature = "cl-aux")]
  ClAux(Box<cl_aux::Error>),
  #[cfg(feature = "crypto-common")]
  CryptoCommonInvalidLength(crypto_common::InvalidLength),
  #[cfg(feature = "base64")]
  DecodeError(base64::DecodeError),
  #[cfg(feature = "base64")]
  DecodeSliceError(base64::DecodeSliceError),
  #[cfg(all(feature = "embassy-net", not(feature = "async-send")))]
  EmbassyNet(embassy_net::tcp::Error),
  #[cfg(feature = "base64")]
  EncodeSliceError(base64::EncodeSliceError),
  #[cfg(feature = "flate2")]
  Flate2CompressError(Box<flate2::CompressError>),
  #[cfg(feature = "flate2")]
  Flate2DecompressError(Box<flate2::DecompressError>),
  #[cfg(all(feature = "glommio", not(feature = "async-send")))]
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
  ArrayStringError(ArrayStringError),
  ArrayVectorError(ArrayVectorError),
  #[allow(private_interfaces)]
  BlocksQueueError(BlocksQueueError),
  #[cfg(feature = "http2")]
  Http2ErrorGoAway(crate::http2::Http2ErrorCode, Option<crate::http2::Http2Error>),
  #[cfg(feature = "http2")]
  Http2ErrorReset(crate::http2::Http2ErrorCode, Option<crate::http2::Http2Error>, u32),
  #[cfg(feature = "orm")]
  OrmError(crate::database::orm::OrmError),
  QueueError(QueueError),
  VectorError(VectorError),
  InvalidHttp2Content,

  /// A slice-like batch of package is not sorted
  CAF_BatchPackagesAreNotSorted,
  /// The server closed the connection
  CAF_ClosedWsConnection,
  /// A server was not able to receive the full request data after several attempts.
  CAF_CouldNotSendTheFullRequestData,
  #[cfg(feature = "client-api-framework")]
  /// GraphQl response error
  CAF_GraphQlResponseError(
    Box<[crate::client_api_framework::data_format::GraphQlResponseError<String>]>,
  ),
  /// The hardware returned an incorrect time value
  CAF_IncorrectHardwareTime,
  /// `no_std` has no knowledge of time.
  CAF_GenericTimeNeedsBackend,
  #[cfg(feature = "client-api-framework")]
  /// JSON-RPC response error
  CAF_JsonRpcResultErr(Box<crate::client_api_framework::data_format::JsonRpcResponseError>),
  /// A given response id is not present in the set of sent packages.
  CAF_ResponseIdIsNotPresentInTheOfSentBatchPackages(usize),
  /// No stored test response to return a result from a request
  CAF_TestTransportNoResponse,
  /// It is not possible to convert a `u16` into a HTTP status code
  CAF_UnknownHttpStatusCode(u16),
  /// `wtx` can not perform this operation due to known limitations.
  CAF_UnsupportedOperation,
  /// Only appending is possible but overwritten is still viable through resetting.
  CAF_UriCanNotBeOverwritten,

  /// The length of a header field must be within a threshold.
  HTTP_HeaderFieldIsTooLarge,
  /// Missing Header
  HTTP_MissingHeader {
    /// See [`KnownHeaderName`].
    expected: KnownHeaderName,
  },
  /// Received request does not contain a method field
  HTTP_MissingRequestMethod,
  /// Received response does not contain a status code field
  HTTP_MissingResponseStatusCode,
  /// HTTP version does not match the expected method.
  HTTP_UnexpectedHttpMethod {
    expected: Method,
  },
  /// Unknown header name.
  HTTP_UnknownHeaderNameFromBytes {
    length: usize,
  },

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
  /// Unexpected end of file when reading.
  MISC_UnexpectedEOF,
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

  /// Not-A-Number is not supported
  PG_DecimalCanNotBeConvertedFromNaN,
  /// Postgres does not support large unsigned integers. For example, `u8` can only be stored
  /// and read with numbers up to 127.
  PG_InvalidPostgresUint,
  /// Received bytes don't compose a valid record.
  PG_InvalidPostgresRecord,
  /// The iterator that composed a `RecordValues` does not contain a corresponding length.
  PG_InvalidRecordValuesIterator,
  /// It is required to connect using a TLS channel but the server didn't provide any. Probably
  /// because the connection is unencrypted.
  PG_MissingChannel,
  /// A "null" field received from the database was decoded as a non-nullable type or value.
  PG_MissingFieldDataInDecoding,
  /// Expected one record but got none.
  PG_NoRecord,
  /// It is required to connect without using a TLS channel but the server only provided a way to
  /// connect using channels. Probably because the connection is encrypted.
  PG_RequiredChannel,
  /// Server does not support encryption
  PG_ServerDoesNotSupportEncryption,
  /// A query
  PG_StatementHashCollision,
  /// Received size differs from expected size.
  PG_UnexpectedBufferSize {
    expected: u64,
    received: u64,
  },
  /// Received an unexpected message type.
  PG_UnexpectedDatabaseMessage {
    received: u8,
  },
  /// Received an expected message type but the related bytes are in an unexpected state.
  PG_UnexpectedDatabaseMessageBytes,
  /// Bytes don't represent expected type
  PG_UnexpectedValueFromBytes {
    expected: &'static str,
  },
  /// The system does not support a requested authentication method.
  PG_UnknownAuthenticationMethod,
  /// The system does not support a provided parameter.
  PG_UnknownConfigurationParameter,
  /// Received a statement ID that is not present in the local cache.
  PG_UnknownStatementId,
  /// The system only supports decimals with 64 digits.
  PG_VeryLargeDecimal,

  /// The `seeds` parameter must be provided through the CLI or the configuration file.
  SM_ChecksumMustBeANumber,
  /// Databases must be sorted and unique
  SM_DatabasesMustBeSortedAndUnique,
  /// Different rollback versions
  SM_DifferentRollbackVersions,
  /// Divergent migrations
  SM_DivergentMigration(i32),
  /// Validation - Migrations number
  SM_DivergentMigrationsNum {
    expected: u32,
    received: u32,
  },
  /// Migration file has invalid syntax,
  SM_InvalidMigration,
  /// TOML parser only supports a subset of the official TOML specification
  SM_TomlParserOnlySupportsStringsAndArraysOfStrings,
  /// TOML parser only supports a subset of the official TOML specification
  SM_TomlValueIsTooLarge,
  /// Migration file has an empty attribute
  SM_IncompleteSqlFile,

  /// It it not possible to read a frame of a connection that was previously closed.
  WS_ConnectionClosed,
  /// HTTP headers must be unique.
  WS_DuplicatedHeader,
  /// The requested received in a handshake on a server is not valid.
  WS_InvalidAcceptRequest,
  /// Received close frame has invalid parameters.
  WS_InvalidCloseFrame,
  /// Received an invalid header compression parameter.
  WS_InvalidCompressionHeaderParameter,
  /// Header indices are out-of-bounds or the number of bytes are too small.
  WS_InvalidFrameHeaderBounds,
  /// Payload indices are out-of-bounds or the number of bytes are too small.
  WS_InvalidPayloadBounds,
  /// Server received a frame without a mask.
  WS_MissingFrameMask,
  /// Client sent "permessage-deflate" but didn't receive back from the server
  WS_MissingPermessageDeflate,
  /// Status code is expected to be
  WS_MissingSwitchingProtocols,
  /// Server responded without a compression context but the client does not allow such behavior.
  WS_NoCompressionContext,
  /// Reserved bits are not zero.
  WS_ReservedBitsAreNotZero,
  /// Received control frame wasn't supposed to be fragmented.
  WS_UnexpectedFragmentedControlFrame,
  /// The first frame of a message is a continuation or the following frames are not a
  /// continuation.
  WS_UnexpectedMessageFrame,
  /// Control frames have a maximum allowed size.
  WS_VeryLargeControlFrame,
  /// Frame payload exceeds the defined threshold.
  WS_VeryLargePayload,
}

impl Display for Error {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    <Self as Debug>::fmt(self, f)
  }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

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
    Self::ClAux(from.into())
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

#[cfg(all(feature = "embassy-net", not(feature = "async-send")))]
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
    Self::Flate2CompressError(from.into())
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

#[cfg(all(feature = "glommio", not(feature = "async-send")))]
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

#[cfg(feature = "orm")]
impl From<crate::database::orm::OrmError> for Error {
  #[inline]
  fn from(from: crate::database::orm::OrmError) -> Self {
    Self::OrmError(from)
  }
}

impl From<QueueError> for Error {
  #[inline]
  fn from(from: QueueError) -> Self {
    Self::QueueError(from)
  }
}

impl From<VectorError> for Error {
  #[inline]
  fn from(from: VectorError) -> Self {
    Self::VectorError(from)
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
