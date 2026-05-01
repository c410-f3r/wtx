#[cfg(feature = "asn1")]
use crate::asn1::Asn1Error;
use crate::{
  calendar::CalendarError,
  codec::{Base64Error, FromRadix10Error, HexError},
  collection::{
    ArrayStringError, ArrayVectorError, BlocksDequeError, DequeueError, FixedStringError,
    ShortBoxStringU16, ShortStrU8, VectorError,
  },
  misc::ArithmeticError,
};
#[allow(unused_imports, reason = "Depends on the selection of features")]
use alloc::boxed::Box;
use alloc::string::String;
use core::{
  alloc::Layout,
  any::Any,
  convert::Infallible,
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
  #[cfg(feature = "argon2")]
  #[doc = associated_element_doc!()]
  Argon2(argon2::Error),
  #[cfg(feature = "embassy-net")]
  #[doc = associated_element_doc!()]
  EmbassyNet(embassy_net::tcp::Error),
  #[cfg(feature = "flate2")]
  #[doc = associated_element_doc!()]
  Flate2CompressError(Box<flate2::CompressError>),
  #[cfg(feature = "flate2")]
  #[doc = associated_element_doc!()]
  Flate2DecompressError(Box<flate2::DecompressError>),
  #[cfg(feature = "getrandom")]
  #[doc = associated_element_doc!()]
  GetRandomError(getrandom::Error),
  #[cfg(feature = "crypto-graviola")]
  #[doc = associated_element_doc!()]
  GraviolaError(graviola::Error),
  #[cfg(feature = "httparse")]
  #[doc = associated_element_doc!()]
  HttpParse(httparse::Error),
  #[cfg(feature = "matchit")]
  #[doc = associated_element_doc!()]
  MatchitError(matchit::MatchError),
  #[cfg(feature = "matchit")]
  #[doc = associated_element_doc!()]
  MatchitInsertError(Box<matchit::InsertError>),
  #[cfg(feature = "crypto-openssl")]
  #[doc = associated_element_doc!()]
  OpensslError(Box<openssl::error::ErrorStack>),
  #[cfg(feature = "quick-protobuf")]
  #[doc = associated_element_doc!()]
  QuickProtobuf(Box<quick_protobuf::Error>),
  #[cfg(feature = "rustls")]
  #[doc = associated_element_doc!()]
  RustlsError(Box<rustls::Error>),
  #[cfg(feature = "serde")]
  #[doc = associated_element_doc!()]
  SerdeDeValue(Box<::serde::de::value::Error>),
  #[cfg(feature = "serde_json")]
  #[doc = associated_element_doc!()]
  SerdeJson(serde_json::Error),
  #[cfg(feature = "serde_json")]
  #[doc = associated_element_doc!()]
  SerdeJsonDeserialize(ShortBoxStringU16),
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
  /// The specified environment variable was not present in the current
  /// process's environment.
  #[cfg(feature = "std")]
  VarIsNotPresent(ShortBoxStringU16),
  /// The specified environment variable was found, but it did not contain
  /// valid unicode data. The found data is returned as a payload of this
  /// variant.
  #[cfg(feature = "std")]
  VarIsNotUnicode(ShortBoxStringU16),
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
  /// Weight is zero, negative or overflowed list
  InvalidWeight,
  /// Future must not be polled again after finalization
  FuturePolledAfterFinalization,
  /// Generic error
  Generic(ShortBoxStringU16),
  /// Generic static error
  GenericStatic(ShortStrU8<'static>),
  /// It is not possible to add an element into an `Option` because it is already occupied.
  InsufficientOptionCapacity,
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
  MissingVar(ShortStrU8<'static>),
  /// A variable does not have an ending quote
  MissingVarQuote(ShortBoxStringU16),
  /// Something prevented a `mlock` operation
  MlockError,
  /// Something prevented a `munlock` operation
  MunlockError,
  /// A variable does not have an ending quote
  NoAvailableVars(ShortBoxStringU16),
  /// Usually used to transform `Option`s into `Result`s
  NoInnerValue(ShortStrU8<'static>),
  /// Byte is not an ASCII graphic character
  NonGraphicByte,
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
    ty: ShortStrU8<'static>,
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
  /// Unsupported operation
  UnsupportedOperation,
  /// Only appending is possible but overwritten is still viable through resetting.
  UriCanNotBeOverwritten,
  /// In the current platform a number is larger than `usize`.
  UsizeConversionOverflow,
  /// It is not possible to write more than 8 slices at once.
  VectoredWriteOverflow,

  // Internal
  //
  #[cfg(feature = "asn1")]
  #[doc = associated_element_doc!()]
  Asn1Error(Asn1Error),
  #[doc = associated_element_doc!()]
  ArithmeticError(ArithmeticError),
  #[doc = associated_element_doc!()]
  ArrayStringError(ArrayStringError),
  #[doc = associated_element_doc!()]
  ArrayVectorError(ArrayVectorError),
  #[doc = associated_element_doc!()]
  Base64Error(Base64Error),
  #[doc = associated_element_doc!()]
  BlocksQueueError(BlocksDequeError),
  #[doc = associated_element_doc!()]
  CalendarError(CalendarError),
  #[cfg(feature = "client-api-framework")]
  #[doc = associated_element_doc!()]
  ClientApiFrameworkError(crate::client_api_framework::ClientApiFrameworkError),
  #[doc = associated_element_doc!()]
  CodecError(Box<crate::codec::CodecError>),
  #[cfg(feature = "http-cookie")]
  #[doc = associated_element_doc!()]
  Cookie(crate::http::CookieError),
  #[cfg(feature = "crypto")]
  #[doc = associated_element_doc!()]
  CryptoError(crate::crypto::CryptoError),
  #[cfg(feature = "database")]
  #[doc = associated_element_doc!()]
  DatabaseError(Box<crate::database::DatabaseError>),
  #[doc = associated_element_doc!()]
  FixedStringError(FixedStringError),
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
  #[doc = associated_element_doc!()]
  VectorError(VectorError),
  #[cfg(feature = "web-socket")]
  #[doc = associated_element_doc!()]
  WebSocketError(crate::web_socket::WebSocketError),
  #[cfg(feature = "x509")]
  #[doc = associated_element_doc!()]
  X509Error(crate::x509::X509Error),
  #[cfg(feature = "x509")]
  #[doc = associated_element_doc!()]
  X509CvError(crate::x509::X509CvError),
}

impl Display for Error {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    <Self as Debug>::fmt(self, f)
  }
}

impl core::error::Error for Error {}

impl From<Infallible> for Error {
  #[inline]
  fn from(value: Infallible) -> Self {
    match value {}
  }
}

impl From<Error> for () {
  #[inline]
  fn from(_: Error) -> Self {}
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

#[cfg(feature = "crypto")]
impl From<crate::crypto::CryptoError> for Error {
  #[inline]
  #[track_caller]
  fn from(from: crate::crypto::CryptoError) -> Self {
    Self::CryptoError(from)
  }
}

#[cfg(feature = "embassy-net")]
impl From<embassy_net::tcp::Error> for Error {
  #[inline]
  fn from(from: embassy_net::tcp::Error) -> Self {
    Self::EmbassyNet(from)
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

#[cfg(feature = "crypto-graviola")]
impl From<graviola::Error> for Error {
  #[inline]
  fn from(from: graviola::Error) -> Self {
    Self::GraviolaError(from)
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

#[cfg(feature = "crypto-openssl")]
impl From<openssl::error::ErrorStack> for Error {
  #[inline]
  fn from(from: openssl::error::ErrorStack) -> Self {
    Self::OpensslError(from.into())
  }
}

#[cfg(feature = "quick-protobuf")]
impl From<quick_protobuf::Error> for Error {
  #[inline]
  fn from(from: quick_protobuf::Error) -> Self {
    Self::QuickProtobuf(from.into())
  }
}

#[cfg(feature = "rustls")]
impl From<rustls::Error> for Error {
  #[inline]
  fn from(from: rustls::Error) -> Self {
    Self::RustlsError(from.into())
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

#[cfg(feature = "http-session")]
impl From<crate::http::SessionError> for Error {
  #[inline]
  fn from(from: crate::http::SessionError) -> Self {
    Self::SessionError(from)
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

impl From<Base64Error> for Error {
  #[inline]
  fn from(from: Base64Error) -> Self {
    Self::Base64Error(from)
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

impl From<crate::codec::CodecError> for Error {
  #[inline]
  fn from(from: crate::codec::CodecError) -> Self {
    Self::CodecError(from.into())
  }
}

#[cfg(feature = "database")]
impl From<crate::database::DatabaseError> for Error {
  #[inline]
  fn from(from: crate::database::DatabaseError) -> Self {
    Self::DatabaseError(from.into())
  }
}

impl From<FixedStringError> for Error {
  #[inline]
  fn from(from: FixedStringError) -> Self {
    Self::FixedStringError(from)
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

#[cfg(feature = "asn1")]
impl From<Asn1Error> for Error {
  #[inline]
  fn from(from: Asn1Error) -> Self {
    Self::Asn1Error(from)
  }
}

impl From<ArithmeticError> for Error {
  #[inline]
  fn from(from: ArithmeticError) -> Self {
    Self::ArithmeticError(from)
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

#[cfg(feature = "x509")]
impl From<crate::x509::X509Error> for Error {
  #[inline]
  fn from(from: crate::x509::X509Error) -> Self {
    Self::X509Error(from)
  }
}

#[cfg(feature = "x509")]
impl From<crate::x509::X509CvError> for Error {
  #[inline]
  fn from(from: crate::x509::X509CvError) -> Self {
    Self::X509CvError(from)
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
      let mut string = msg.to_string();
      string.truncate(1024);
      Self::Generic(string.try_into().unwrap_or_default())
    }
  }
}
