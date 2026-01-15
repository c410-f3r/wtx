use crate::{
  calendar::CalendarError,
  collection::{
    ArrayStringError, ArrayStringU8, ArrayVectorError, BlocksDequeError, DequeueError, VectorError,
  },
  de::{FromRadix10Error, HexError},
};
#[allow(unused_imports, reason = "Depends on the selection of features")]
use alloc::boxed::Box;
use alloc::string::String;
use core::{
  alloc::Layout,
  any::Any,
  fmt::{Debug, Display, Formatter, Write as _},
  ops::RangeInclusive,
};

#[cfg(target_pointer_width = "64")]
const _: () = {
  assert!(size_of::<Error>() <= 16);
};

macro_rules! associated_element_doc {
  () => {
    "See the documentation of the associated element."
  };
}

/// Grouped individual errors
#[derive(Debug)]
pub enum Error {
  // External - Third parties
  //
  #[cfg(feature = "aes-gcm")]
  #[doc = associated_element_doc!()]
  AeadError(aes_gcm::aead::Error),
  #[cfg(feature = "argon2")]
  #[doc = associated_element_doc!()]
  Argon2(argon2::Error),
  #[cfg(feature = "crypto-common")]
  #[doc = associated_element_doc!()]
  CryptoCommonInvalidLength(crypto_common::InvalidLength),
  #[cfg(feature = "base64")]
  #[doc = associated_element_doc!()]
  DecodeError(Box<base64::DecodeError>),
  #[cfg(feature = "base64")]
  #[doc = associated_element_doc!()]
  DecodeSliceError(Box<base64::DecodeSliceError>),
  #[cfg(feature = "embassy-net")]
  #[doc = associated_element_doc!()]
  EmbassyNet(embassy_net::tcp::Error),
  #[cfg(feature = "base64")]
  #[doc = associated_element_doc!()]
  EncodeSliceError(base64::EncodeSliceError),
  #[cfg(feature = "flate2")]
  #[doc = associated_element_doc!()]
  Flate2CompressError(Box<flate2::CompressError>),
  #[cfg(feature = "flate2")]
  #[doc = associated_element_doc!()]
  Flate2DecompressError(Box<flate2::DecompressError>),
  #[cfg(feature = "getrandom")]
  #[doc = associated_element_doc!()]
  GetRandomError(getrandom::Error),
  #[cfg(feature = "httparse")]
  #[doc = associated_element_doc!()]
  HttpParse(httparse::Error),
  #[cfg(feature = "matchit")]
  #[doc = associated_element_doc!()]
  MatchitError(matchit::MatchError),
  #[cfg(feature = "matchit")]
  #[doc = associated_element_doc!()]
  MatchitInsertError(Box<matchit::InsertError>),
  #[cfg(feature = "digest")]
  #[doc = associated_element_doc!()]
  MacError(digest::MacError),
  #[cfg(feature = "quick-protobuf")]
  #[doc = associated_element_doc!()]
  QuickProtobuf(Box<quick_protobuf::Error>),
  #[cfg(feature = "rsa")]
  #[doc = associated_element_doc!()]
  RsaError(Box<rsa::Error>),
  #[cfg(feature = "rustls-webpki")]
  #[doc = associated_element_doc!()]
  RustlsWebpki(Box<webpki::Error>),
  #[cfg(feature = "serde")]
  #[doc = associated_element_doc!()]
  SerdeDeValue(Box<::serde::de::value::Error>),
  #[cfg(feature = "serde_json")]
  #[doc = associated_element_doc!()]
  SerdeJson(serde_json::Error),
  #[cfg(feature = "serde_json")]
  #[doc = associated_element_doc!()]
  SerdeJsonDeserialize(Box<String>),
  #[cfg(feature = "serde_urlencoded")]
  #[doc = associated_element_doc!()]
  SerdeUrlencodedSer(Box<serde_urlencoded::ser::Error>),
  #[cfg(feature = "spki")]
  #[doc = associated_element_doc!()]
  SpkiError(Box<spki::Error>),
  #[cfg(feature = "tokio")]
  #[doc = associated_element_doc!()]
  TokioJoinError(Box<tokio::task::JoinError>),
  #[cfg(feature = "tracing-subscriber")]
  #[doc = associated_element_doc!()]
  TryInitError(Box<tracing_subscriber::util::TryInitError>),
  #[cfg(feature = "std")]
  #[doc = associated_element_doc!()]
  TryLockError(std::sync::TryLockError<()>),
  #[cfg(feature = "uuid")]
  #[doc = associated_element_doc!()]
  UuidError(Box<uuid::Error>),

  // External - Std
  //
  #[doc = associated_element_doc!()]
  AddrParseError(core::net::AddrParseError),
  #[doc = associated_element_doc!()]
  Fmt(core::fmt::Error),
  #[cfg(feature = "std")]
  #[doc = associated_element_doc!()]
  IoError(std::io::Error),
  #[doc = associated_element_doc!()]
  ParseIntError(core::num::ParseIntError),
  #[doc = associated_element_doc!()]
  RecvError(RecvError),
  #[doc = associated_element_doc!()]
  SendError(SendError<()>),
  #[doc = associated_element_doc!()]
  TryFromIntError(core::num::TryFromIntError),
  #[doc = associated_element_doc!()]
  TryFromSliceError(core::array::TryFromSliceError),
  #[cfg(feature = "std")]
  #[doc = associated_element_doc!()]
  VarError(VarError),
  #[doc = associated_element_doc!()]
  Utf8Error(Box<core::str::Utf8Error>),

  // Generic
  //
  /// Allocation error
  AllocError(Box<Layout>),
  /// A connection was unexpectedly closed by an external actor or because of a local error.
  ClosedConnection,
  /// A HTTP connection was unexpectedly closed by an external actor or because of a local error.
  ClosedHttpConnection,
  /// A WebSocket connection was unexpectedly closed by an external actor or because of a local error.
  ClosedWebSocketConnection,
  /// The amount of elements exceeds the underlying storage.
  CounterWriterOverflow,
  /// Future should complete before a certain duration but didn't
  ExpiredFuture,
  /// Future must not be polled again after finalization
  FuturePolledAfterFinalization,
  /// Generic error
  Generic(Box<String>),
  /// It is not possible to add an element into an `Option` because it is already occupied.
  InsufficientOptionCapacity,
  /// Data must have a nonce and a tag.
  InvalidAes256GcmData,
  /// Indices are out-of-bounds or the number of bytes are too small.
  InvalidPartitionedBufferBounds,
  /// Invalid UTF-8.
  InvalidUTF8,
  /// An index that cuts an UTF-8 string makes the sequence invalid.
  InvalidUTF8Bound,
  /// Invalid URI
  InvalidUri,
  /// There is no CA provider.
  MissingCaProviders,
  /// A instance could not be constructed because of a missing required variable.
  MissingVar(Box<&'static str>),
  /// A variable does not have an ending quote
  MissingVarQuote(Box<String>),
  /// Something prevented a `mlock` operation
  MlockError,
  /// Something prevented a `munlock` operation
  MunlockError,
  /// A variable does not have an ending quote
  NoAvailableVars(Box<String>),
  /// Usually used to transform `Option`s into `Result`s
  NoInnerValue(Box<&'static str>),
  /// A set of arithmetic operations resulted in an overflow, underflow or division by zero
  OutOfBoundsArithmetic,
  /// An error that shouldn't exist. If this variant is raised, then it is very likely that the
  /// involved code was not built the way it should be.
  ProgrammingError,
  /// Unexpected Unsigned integer
  UnboundedNumber {
    /// Expected bounds
    expected: RangeInclusive<i32>,
    /// Received number
    received: i32,
  },
  /// A buffer was partially read or write but should in fact be fully processed.
  UnexpectedBufferState,
  /// Unexpected bytes
  UnexpectedBytes {
    /// Length of the unexpected bytes
    length: u16,
    /// Name of the associated entity
    ty: ArrayStringU8<9>,
  },
  /// Unexpected end of file when reading from a stream.
  UnexpectedStreamReadEOF,
  /// Unexpected end of file when writing to a stream.
  UnexpectedStreamWriteEOF,
  /// Unexpected string
  UnexpectedString {
    /// Length of the unexpected string
    length: usize,
  },
  /// Unexpected Unsigned integer
  UnexpectedUint {
    /// Number value
    received: u64,
  },
  /// The operation `mlock` is not supported in your platform
  UnsupportedMlockPlatform,
  /// Only appending is possible but overwritten is still viable through resetting.
  UriCanNotBeOverwritten,
  /// In the current platform a number is larger than `usize`.
  UsizeConversionOverflow,
  /// It is not possible to write more than 8 slices at once.
  VectoredWriteOverflow,

  // Internal
  //
  #[doc = associated_element_doc!()]
  ArrayStringError(ArrayStringError),
  #[doc = associated_element_doc!()]
  ArrayVectorError(ArrayVectorError),
  #[cfg(feature = "aws-lc-rs")]
  #[doc = associated_element_doc!()]
  AwsLcRsUnspecified(aws_lc_rs::error::Unspecified),
  #[doc = associated_element_doc!()]
  BlocksQueueError(BlocksDequeError),
  #[doc = associated_element_doc!()]
  CalendarError(CalendarError),
  #[cfg(feature = "client-api-framework")]
  #[doc = associated_element_doc!()]
  ClientApiFrameworkError(crate::client_api_framework::ClientApiFrameworkError),
  #[cfg(feature = "http-cookie")]
  #[doc = associated_element_doc!()]
  Cookie(crate::http::CookieError),
  #[cfg(feature = "database")]
  #[doc = associated_element_doc!()]
  DatabaseError(Box<crate::database::DatabaseError>),
  #[doc = associated_element_doc!()]
  DecError(crate::de::DecError),
  #[doc = associated_element_doc!()]
  FromRadix10Error(FromRadix10Error),
  #[doc = associated_element_doc!()]
  HexError(HexError),
  #[cfg(feature = "http")]
  #[doc = associated_element_doc!()]
  HttpError(crate::http::HttpError),
  #[cfg(feature = "http2")]
  #[doc = associated_element_doc!()]
  Http2ErrorGoAway(crate::http2::Http2ErrorCode, crate::http2::Http2Error),
  #[cfg(feature = "http2")]
  #[doc = associated_element_doc!()]
  Http2FlowControlError(crate::http2::Http2Error, u32),
  #[cfg(feature = "mysql")]
  #[doc = associated_element_doc!()]
  MysqlDbError(Box<crate::database::client::mysql::DbError>),
  #[cfg(feature = "mysql")]
  #[doc = associated_element_doc!()]
  MysqlError(crate::database::client::mysql::MysqlError),
  #[cfg(feature = "postgres")]
  #[doc = associated_element_doc!()]
  PostgresDbError(Box<crate::database::client::postgres::DbError>),
  #[cfg(feature = "postgres")]
  #[doc = associated_element_doc!()]
  PostgresError(crate::database::client::postgres::PostgresError),
  #[doc = associated_element_doc!()]
  QueueError(DequeueError),
  #[cfg(feature = "schema-manager")]
  #[doc = associated_element_doc!()]
  SchemaManagerError(crate::database::schema_manager::SchemaManagerError),
  #[cfg(feature = "http-server-framework")]
  #[doc = associated_element_doc!()]
  ServerFrameworkError(crate::http::server_framework::ServerFrameworkError),
  #[cfg(feature = "http-session")]
  #[doc = associated_element_doc!()]
  SessionError(crate::http::SessionError),
  #[cfg(feature = "tls")]
  #[doc = associated_element_doc!()]
  TlsError(crate::tls::TlsError),
  #[doc = associated_element_doc!()]
  VectorError(VectorError),
  #[cfg(feature = "web-socket")]
  #[doc = associated_element_doc!()]
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

#[cfg(feature = "aes-gcm")]
impl From<aes_gcm::aead::Error> for Error {
  #[inline]
  #[track_caller]
  fn from(from: aes_gcm::aead::Error) -> Self {
    Self::AeadError(from)
  }
}

#[cfg(feature = "argon2")]
impl From<argon2::Error> for Error {
  #[inline]
  #[track_caller]
  fn from(from: argon2::Error) -> Self {
    Self::Argon2(from)
  }
}

#[cfg(feature = "http-cookie")]
impl From<crate::http::CookieError> for Error {
  #[inline]
  #[track_caller]
  fn from(from: crate::http::CookieError) -> Self {
    Self::Cookie(from)
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
    Self::DecodeError(from.into())
  }
}

#[cfg(feature = "base64")]
impl From<base64::DecodeSliceError> for Error {
  #[inline]
  #[track_caller]
  fn from(from: base64::DecodeSliceError) -> Self {
    Self::DecodeSliceError(from.into())
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

#[cfg(feature = "getrandom")]
impl From<getrandom::Error> for Error {
  #[inline]
  fn from(from: getrandom::Error) -> Self {
    Self::GetRandomError(from)
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

#[cfg(feature = "matchit")]
impl From<matchit::MatchError> for Error {
  #[inline]
  fn from(from: matchit::MatchError) -> Self {
    Self::MatchitError(from)
  }
}

#[cfg(feature = "matchit")]
impl From<matchit::InsertError> for Error {
  #[inline]
  fn from(from: matchit::InsertError) -> Self {
    Self::MatchitInsertError(from.into())
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

#[cfg(feature = "mysql")]
impl From<crate::database::client::mysql::DbError> for Error {
  #[inline]
  fn from(from: crate::database::client::mysql::DbError) -> Self {
    Self::MysqlDbError(from.into())
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

#[cfg(feature = "rsa")]
impl From<rsa::Error> for Error {
  #[inline]
  fn from(from: rsa::Error) -> Self {
    Self::RsaError(from.into())
  }
}

#[cfg(feature = "rustls-webpki")]
impl From<webpki::Error> for Error {
  #[inline]
  fn from(from: webpki::Error) -> Self {
    Self::RustlsWebpki(from.into())
  }
}

#[cfg(feature = "serde")]
impl From<::serde::de::value::Error> for Error {
  #[inline]
  fn from(from: ::serde::de::value::Error) -> Self {
    Self::SerdeDeValue(from.into())
  }
}

#[cfg(feature = "serde_json")]
impl From<serde_json::Error> for Error {
  #[inline]
  fn from(from: serde_json::Error) -> Self {
    Self::SerdeJson(from)
  }
}

#[cfg(feature = "serde_urlencoded")]
impl From<serde_urlencoded::ser::Error> for Error {
  #[inline]
  fn from(from: serde_urlencoded::ser::Error) -> Self {
    Self::SerdeUrlencodedSer(from.into())
  }
}

#[cfg(feature = "http-session")]
impl From<crate::http::SessionError> for Error {
  #[inline]
  fn from(from: crate::http::SessionError) -> Self {
    Self::SessionError(from)
  }
}

#[cfg(feature = "spki")]
impl From<spki::Error> for Error {
  #[inline]
  fn from(from: spki::Error) -> Self {
    Self::SpkiError(from.into())
  }
}

#[cfg(feature = "tls")]
impl From<crate::tls::TlsError> for Error {
  #[inline]
  fn from(from: crate::tls::TlsError) -> Self {
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

#[cfg(feature = "tracing-subscriber")]
impl From<tracing_subscriber::util::TryInitError> for Error {
  #[inline]
  fn from(from: tracing_subscriber::util::TryInitError) -> Self {
    Self::TryInitError(from.into())
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

#[cfg(feature = "uuid")]
impl From<uuid::Error> for Error {
  #[inline]
  fn from(value: uuid::Error) -> Self {
    Self::UuidError(value.into())
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

#[cfg(feature = "aws-lc-rs")]
impl From<aws_lc_rs::error::Unspecified> for Error {
  #[inline]
  fn from(from: aws_lc_rs::error::Unspecified) -> Self {
    Self::AwsLcRsUnspecified(from)
  }
}

impl From<BlocksDequeError> for Error {
  #[inline]
  fn from(from: BlocksDequeError) -> Self {
    Self::BlocksQueueError(from)
  }
}

impl From<CalendarError> for Error {
  #[inline]
  fn from(from: CalendarError) -> Self {
    Self::CalendarError(from)
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
    Self::DatabaseError(from.into())
  }
}

impl From<crate::de::DecError> for Error {
  #[inline]
  fn from(from: crate::de::DecError) -> Self {
    Self::DecError(from)
  }
}

impl From<FromRadix10Error> for Error {
  #[inline]
  fn from(from: FromRadix10Error) -> Self {
    Self::FromRadix10Error(from)
  }
}

impl From<HexError> for Error {
  #[inline]
  fn from(from: HexError) -> Self {
    Self::HexError(from)
  }
}

#[cfg(feature = "mysql")]
impl From<crate::database::client::mysql::MysqlError> for Error {
  #[inline]
  fn from(from: crate::database::client::mysql::MysqlError) -> Self {
    Self::MysqlError(from)
  }
}

#[cfg(feature = "postgres")]
impl From<crate::database::client::postgres::PostgresError> for Error {
  #[inline]
  fn from(from: crate::database::client::postgres::PostgresError) -> Self {
    Self::PostgresError(from)
  }
}

impl From<DequeueError> for Error {
  #[inline]
  fn from(from: DequeueError) -> Self {
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

#[cfg(feature = "http-server-framework")]
impl From<crate::http::server_framework::ServerFrameworkError> for Error {
  #[inline]
  fn from(from: crate::http::server_framework::ServerFrameworkError) -> Self {
    Self::ServerFrameworkError(from)
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

impl From<Box<dyn Any + Send + 'static>> for Error {
  #[inline]
  fn from(from: Box<dyn Any + Send + 'static>) -> Self {
    let mut string = String::new();
    let _rslt = string.write_fmt(format_args!("{from:?}"));
    Self::Generic(Box::new(string))
  }
}

/// An error returned by the receiving part of a channel
#[derive(Debug)]
pub enum RecvError {
  /// A message could not be received because the channel is empty.
  Empty,
  /// The message could not be received because the channel is empty and disconnected.
  Disconnected,
}

/// An error returned by the sending part of a channel
#[derive(Debug)]
pub enum SendError<T> {
  /// The message could not be sent because the channel is full.
  Full(T),
  /// The message could not be sent because the channel is disconnected.
  Disconnected(T),
}

impl<T> SendError<T> {
  /// Removes the inner element
  #[inline]
  pub fn simplify(self) -> SendError<()> {
    match self {
      SendError::Full(_) => SendError::Full(()),
      SendError::Disconnected(_) => SendError::Disconnected(()),
    }
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

#[cfg(feature = "serde")]
mod serde {
  use alloc::string::ToString;
  use core::fmt::Display;

  impl serde::ser::Error for crate::Error {
    #[inline]
    fn custom<T>(msg: T) -> Self
    where
      T: Display,
    {
      Self::Generic(msg.to_string().into())
    }
  }
}
