use crate::http::ExpectedHeader;
use core::{
  fmt::{Debug, Display, Formatter},
  ops::RangeInclusive,
};
#[allow(unused_imports)]
use {alloc::boxed::Box, alloc::string::String};

#[cfg(target_pointer_width = "64")]
const _: () = {
  assert!(core::mem::size_of::<Error>() == 24);
};

#[cfg(feature = "rkyv")]
type RkyvSer = rkyv::ser::serializers::CompositeSerializerError<
  core::convert::Infallible,
  rkyv::ser::serializers::AllocScratchError,
  rkyv::ser::serializers::SharedSerializeMapError,
>;

/// Grouped individual errors
//
// * `Invalid` Something is present but has invalid state.
// * `Missing`: Not present when expected to be.
// * `Unexpected`: Received something that was not intended.
#[allow(missing_docs)]
#[derive(Debug)]
pub enum Error {
  // External - Misc
  //
  #[cfg(feature = "arrayvec")]
  ArrayVec(arrayvec::CapacityError<()>),
  AtoiInvalidBytes,
  CapacityOverflow,
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
  #[cfg(feature = "embassy-net")]
  EmbassyNet(embassy_net::tcp::Error),
  #[cfg(feature = "base64")]
  EncodeSliceError(base64::EncodeSliceError),
  #[cfg(feature = "flate2")]
  Flate2CompressError(Box<flate2::CompressError>),
  #[cfg(feature = "flate2")]
  Flate2DecompressError(Box<flate2::DecompressError>),
  #[cfg(feature = "glommio")]
  Glommio(Box<glommio::GlommioError<()>>),
  #[cfg(feature = "httparse")]
  HttpParse(httparse::Error),
  #[cfg(feature = "http2")]
  Http2ErrorCode(crate::http2::ErrorCode),
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
  #[cfg(feature = "std")]
  AddrParseError(std::net::AddrParseError),
  Fmt(core::fmt::Error),
  #[cfg(feature = "std")]
  IoError(std::io::Error),
  ParseIntError(core::num::ParseIntError),
  TryFromIntError(core::num::TryFromIntError),
  TryFromSliceError(core::array::TryFromSliceError),
  #[cfg(feature = "std")]
  VarError(VarError),
  Utf8Error(core::str::Utf8Error),

  // ***** Internal - Client API Framework *****
  //
  /// A slice-like batch of package is not sorted
  BatchPackagesAreNotSorted,
  /// The server closed the WebSocket connection
  ClosedWsConnection,
  /// A server was not able to receive the full request data after several attempts.
  CouldNotSendTheFullRequestData,
  #[cfg(feature = "client-api-framework")]
  /// GraphQl response error
  GraphQlResponseError(
    Box<[crate::client_api_framework::data_format::GraphQlResponseError<String>]>,
  ),
  /// The hardware returned an incorrect time value
  IncorrectHardwareTime,
  /// `no_std` has no knowledge of time. Try enabling the `std` feature
  ItIsNotPossibleToUseTimeInNoStd,
  #[cfg(feature = "client-api-framework")]
  /// JSON-RPC response error
  JsonRpcResultErr(Box<crate::client_api_framework::data_format::JsonRpcResponseError>),
  /// A variant used to transform `Option`s into `Result`s
  NoInnerValue(&'static str),
  /// A given response id is not present in the set of sent packages.
  ResponseIdIsNotPresentInTheOfSentBatchPackages(usize),
  /// No stored test response to return a result from a request
  TestTransportNoResponse,
  /// It is not possible to convert a `u16` into a HTTP status code
  UnknownHttpStatusCode(u16),
  /// `wtx` can not perform this operation due to known limitations.
  UnsupportedOperation,
  /// Only appending is possible but overwritten is still viable through resetting.
  UriCanNotBeOverwritten,

  // ***** Internal - Database client *****
  //
  /// A "null" field received from the database was decoded as a non-nullable type or value.
  MissingFieldDataInDecoding,
  /// Not-A-Number is not supported
  DecimalCanNotBeConvertedFromNaN,
  /// Postgres does not support large unsigned integers. For example, `u8` can only be stored
  /// and read with numbers up to 127.
  InvalidPostgresUint,
  /// Received bytes don't compose a valid record.
  InvalidPostgresRecord,
  /// The iterator that composed a `RecordValues`` does not contain a corresponding length.
  InvalidRecordValuesIterator,
  /// Expected one record but got none.
  NoRecord,
  /// A query
  StatementHashCollision,
  /// Received size differs from expected size.
  UnexpectedBufferSize {
    expected: u64,
    received: u64,
  },
  /// Received an unexpected message type.
  UnexpectedDatabaseMessage {
    received: u8,
  },
  /// Received an expected message type but the related bytes are in an unexpected state.
  UnexpectedDatabaseMessageBytes,
  /// Bytes don't represent expected type
  UnexpectedValueFromBytes {
    expected: &'static str,
  },
  /// The system does not support a requested authentication method.
  UnknownAuthenticationMethod,
  /// The system does not support a provided parameter.
  UnknownConfigurationParameter,
  /// Received a statement ID that is not present in the local cache.
  UnknownStatementId,
  /// The system only supports decimals with 64 digits.
  VeryLargeDecimal,

  // ***** Internal - Database SM *****
  //
  /// The `seeds` parameter must be provided through the CLI or the configuration file.
  ChecksumMustBeANumber,
  /// Databases must be sorted and unique
  DatabasesMustBeSortedAndUnique,
  /// Different rollback versions
  DifferentRollbackVersions,
  /// Divergent migrations
  DivergentMigration(i32),
  /// Validation - Migrations number
  DivergentMigrationsNum {
    expected: u32,
    received: u32,
  },
  /// Migration file has invalid syntax,
  InvalidMigration,
  /// TOML parser only supports a subset of the official TOML specification
  TomlParserOnlySupportsStringsAndArraysOfStrings,
  /// TOML parser only supports a subset of the official TOML specification
  TomlValueIsTooLarge,

  // ***** Internal - Database ORM *****
  //
  /// Migration file has an empty attribute
  IncompleteSqlFile,
  /// Some internal operation found a hash collision of two table ids (likely) or a hash collision
  /// due to a number of nested associations larger than `MAX_NODES_NUM` (unlikely).
  TableHashCollision(&'static str),

  // ***** Internal - Generic *****
  //
  /// Invalid UTF-8.
  InvalidUTF8,
  /// Indices are out-of-bounds or the number of bytes are too small.
  InvalidPartitionedBufferBounds,
  /// An expected value could not be found
  InvalidDatabaseUrl(&'static str),
  /// Backend couldn't perform passed query string
  InvalidSqlQuery,
  /// Invalid URL
  InvalidUrl,
  /// Environment variable is not present
  MissingEnvVar,
  /// A set of arithmetic operations resulted in an overflow, underflow or division by zero
  OutOfBoundsArithmetic,
  /// Unexpected String
  UnexpectedString {
    length: usize,
  },
  /// Unexpected Unsigned integer
  UnexpectedUint {
    received: u32,
  },
  /// Unexpected Unsigned integer
  UnboundedNumber {
    expected: RangeInclusive<u32>,
    received: u32,
  },

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
  /// Server does not support encryption
  ServerDoesNotSupportEncryption,
  /// Stream does not support TLS channels.
  StreamDoesNotSupportTlsChannels,

  // ***** Internal - HTTP *****
  //
  /// The length of a header field must be within a threshold.
  HeaderFieldIsTooLarge,
  /// Received Request does not contain a method field
  MissingRequestMethod,
  /// Received Response does not contain a status code field
  MissingResponseStatusCode,
  /// Unknown header name.
  UnknownHeaderName,

  // ***** Internal - HTTP/2 *****
  //
  /// Unknown header name.
  UnexpectedPreFixedHeaderName,
  /// Decoding logic encountered an unexpected ending string signal.
  UnexpectedEndingHuffman,
  /// A container does not contain an element referred by the given idx
  InvalidHpackIdx(usize),
  /// Header integers must be equal or lesser than `u16::MAX`
  VeryLargeHeaderInteger,
  /// Size updates of dynamic table can't be placed after the first header
  InvalidDynTableSizeUpdate,
  /// Length of a header name or value is limited to 127 bytes.
  UnsupportedHeaderNameOrValueLen,
  /// Received an Hpack index that does not adhere to the standard
  UnexpectedHpackIdx,
  /// Type is out of range or unsupported.
  UnknownSettingFrameTy,
  /// Settings frame identifier is not zero
  UnexpectedSettingsIdentifier,
  /// Counter-part did not return the correct bytes of a HTTP2 connection preface
  NoPreface,
  #[doc = concat!(
    "The system does not support more than",
    _max_continuation_frames!(),
    " continuation frames."
  )]
  VeryLargeAmountOfContinuationFrames,
  #[doc = concat!(
    "The system does not support more than",
    _max_frames_mismatches!(),
    " fetches of frames with mismatches IDs or mismatches types"
  )]
  VeryLargeAmountOfFrameMismatches,
  /// Frames can not be greater than
  VeryLargeFrame,
  /// Endpoint didn't send an ACK response
  NoAckSettings,
  /// Received a continuation or data frame instead of a header frame.
  NotAInitialHeaderFrame,
  /// Received a stream ID that doesn't exist locally
  UnknownStreamId,
  VeryLargeAmountOfBufferedFrames,
  ExceedAmountOfRapidResets,
  ExceedAmountOfActiveConcurrentStreams,
  VeryLargeHeadersLen,
  WindowSizeCanNotBeReduced,
  InvalidStreamState,

  // ***** Internal - WebSocket *****
  //
  /// The requested received in a handshake on a server is not valid.
  InvalidAcceptRequest,
  /// Received close frame has invalid parameters.
  InvalidCloseFrame,
  /// Received an invalid header compression parameter.
  InvalidCompressionHeaderParameter,
  /// Header indices are out-of-bounds or the number of bytes are too small.
  InvalidFrameHeaderBounds,
  /// Payload indices are out-of-bounds or the number of bytes are too small.
  InvalidPayloadBounds,

  /// Server received a frame without a mask.
  MissingFrameMask,
  /// Client sent "permessage-deflate" but didn't receive back from the server
  MissingPermessageDeflate,
  /// Status code is expected to be
  MissingSwitchingProtocols,

  /// Received control frame wasn't supposed to be fragmented.
  UnexpectedFragmentedControlFrame,
  /// The first frame of a message is a continuation or the following frames are not a
  /// continuation.
  UnexpectedMessageFrame,

  /// It it not possible to read a frame of a connection that was previously closed.
  ConnectionClosed,
  /// Server responded without a compression context but the client does not allow such behavior.
  NoCompressionContext,
  /// Reserved bits are not zero.
  ReservedBitsAreNotZero,
  /// Control frames have a maximum allowed size.
  VeryLargeControlFrame,
  /// Frame payload exceeds the defined threshold.
  VeryLargePayload,
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

#[cfg(feature = "arrayvec")]
impl<T> From<arrayvec::CapacityError<T>> for Error {
  #[inline]
  #[track_caller]
  fn from(from: arrayvec::CapacityError<T>) -> Self {
    Self::ArrayVec(from.simplify())
  }
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

#[cfg(feature = "std")]
impl From<std::net::AddrParseError> for Error {
  #[inline]
  fn from(from: std::net::AddrParseError) -> Self {
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

#[cfg(feature = "http2")]
impl From<crate::http2::ErrorCode> for Error {
  #[inline]
  fn from(from: crate::http2::ErrorCode) -> Self {
    Self::Http2ErrorCode(from)
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
