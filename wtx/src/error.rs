mod external_std;
mod external_third_parties;
mod internal;

#[cfg(feature = "asn1")]
use crate::asn1::Asn1Error;
use crate::{
  calendar::CalendarError,
  codec::{Base64Error, FromRadix10Error, HexError},
  collections::{
    ArrayStringError, ArrayVectorError, BlocksDequeError, DequeueError, FixedStringError,
    ShortBoxStrU16, ShortStrU8, VectorError,
  },
  misc::{ArithmeticError, AsciiError},
};
#[allow(unused_imports, reason = "Depends on the selection of features")]
use alloc::boxed::Box;
use core::{
  alloc::Layout,
  array::TryFromSliceError,
  convert::Infallible,
  fmt::{Debug, Display, Formatter},
  ops::RangeInclusive,
  slice::GetDisjointMutError,
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
  #[cfg(feature = "crypto-ruco")]
  #[doc = associated_element_doc!()]
  AeadError(aead::Error),
  #[cfg(feature = "argon2")]
  #[doc = associated_element_doc!()]
  Argon2(argon2::Error),
  #[cfg(feature = "crypto-ruco")]
  #[doc = associated_element_doc!()]
  CryptoCommonInvalidLength(crypto_common::InvalidLength),
  #[cfg(feature = "crypto-ruco")]
  #[doc = associated_element_doc!()]
  EllipticCurveError(elliptic_curve::Error),
  #[cfg(feature = "embassy-net")]
  #[doc = associated_element_doc!()]
  EmbassyNetTcp(embassy_net::tcp::Error),
  #[cfg(feature = "embassy-net")]
  #[doc = associated_element_doc!()]
  EmbassyNetUdpBind(embassy_net::udp::BindError),
  #[cfg(feature = "embassy-net")]
  #[doc = associated_element_doc!()]
  EmbassyNetUdpRecv(embassy_net::udp::RecvError),
  #[cfg(feature = "embassy-net")]
  #[doc = associated_element_doc!()]
  EmbassyNetUdpSend(embassy_net::udp::SendError),
  #[cfg(feature = "getrandom")]
  #[doc = associated_element_doc!()]
  GetRandomError(getrandom::Error),
  #[cfg(feature = "crypto-graviola")]
  #[doc = associated_element_doc!()]
  GraviolaError(graviola::Error),
  #[cfg(feature = "httparse")]
  #[doc = associated_element_doc!()]
  HttpParse(httparse::Error),
  #[cfg(feature = "crypto-ruco")]
  #[doc = associated_element_doc!()]
  MacError(digest::MacError),
  #[cfg(feature = "crypto-ruco")]
  #[doc = associated_element_doc!()]
  Pkcs8Error(Box<pkcs8::Error>),
  #[cfg(feature = "quick-protobuf")]
  #[doc = associated_element_doc!()]
  QuickProtobuf(Box<quick_protobuf::Error>),
  #[cfg(feature = "crypto-ruco")]
  #[doc = associated_element_doc!()]
  RsaError(Box<rsa::Error>),
  #[cfg(feature = "serde")]
  #[doc = associated_element_doc!()]
  SerdeDeValue(Box<::serde::de::value::Error>),
  #[cfg(feature = "serde_json")]
  #[doc = associated_element_doc!()]
  SerdeJson(serde_json::Error),
  #[cfg(feature = "serde_json")]
  #[doc = associated_element_doc!()]
  SerdeJsonDeserialize(ShortBoxStrU16),
  #[cfg(feature = "crypto-ruco")]
  #[doc = associated_element_doc!()]
  Signature(Box<signature::Error>),
  #[cfg(feature = "crypto-ruco")]
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
  #[cfg(feature = "zlib-rs")]
  #[doc = associated_element_doc!()]
  ZlibRsDeflateError(zlib_rs::DeflateError),
  #[cfg(feature = "zlib-rs")]
  #[doc = associated_element_doc!()]
  ZlibRsInflateError(zlib_rs::InflateError),

  // External - Std
  //
  #[doc = associated_element_doc!()]
  AddrParseError(core::net::AddrParseError),
  #[doc = associated_element_doc!()]
  Fmt(core::fmt::Error),
  #[doc = associated_element_doc!()]
  GetDisjointMutError(GetDisjointMutError),
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
  TryFromSliceError(TryFromSliceError),
  #[doc = associated_element_doc!()]
  Utf8Error(Box<core::str::Utf8Error>),
  /// The specified environment variable was not present in the current
  /// process's environment.
  #[cfg(feature = "std")]
  VarIsNotPresent(ShortBoxStrU16),
  /// The specified environment variable was found, but it did not contain
  /// valid unicode data. The found data is returned as a payload of this
  /// variant.
  #[cfg(feature = "std")]
  VarIsNotUnicode(ShortBoxStrU16),

  // Generic
  //
  /// Allocation error
  AllocError(Box<Layout>),
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
  Generic(ShortBoxStrU16),
  /// Generic static error
  GenericStatic(ShortStrU8<'static>),
  /// It is not possible to add an element into an `Option` because it is already occupied.
  InsufficientOptionCapacity,
  /// Indices are out-of-bounds or the number of bytes are too small.
  InvalidPartitionedBufferBounds,
  /// Invalid PPM value
  InvalidPpmValue,
  /// Invalid UTF-8.
  InvalidUTF8,
  /// An index that cuts an UTF-8 string makes the sequence invalid.
  InvalidUTF8Bound,
  /// Invalid URI
  InvalidUri,
  /// Weight is zero, negative or overflowed list
  InvalidWeight,
  /// There is no CA provider.
  MissingCaProviders,
  /// A instance could not be constructed because of a missing required variable.
  MissingVar(ShortStrU8<'static>),
  /// A variable does not have an ending quote
  MissingVarQuote(ShortBoxStrU16),
  /// Something prevented a `mlock` operation
  MlockError,
  /// Something prevented a `munlock` operation
  MunlockError,
  /// A variable does not have an ending quote
  NoAvailableVars(ShortBoxStrU16),
  /// Usually used to transform `Option`s into `Result`s
  NoInnerValue(ShortStrU8<'static>),
  /// Byte is not an ASCII graphic character
  NonGraphicByte,
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
  /// In compression/decompression. A buffer was partially read or write but should in fact be
  /// fully processed.
  UnexpectedBufferStateInCodec,
  /// Unexpected bytes
  UnexpectedBytes {
    /// Length of the unexpected bytes
    length: u16,
    /// Name of the associated entity
    ty: ShortStrU8<'static>,
  },
  /// Unexpected string
  UnexpectedString {
    /// Length of the unexpected string
    length: usize,
  },
  /// Unexpected Unsigned integer
  UnexpectedUint {
    /// Identifier
    identifier: ShortStrU8<'static>,
    /// Number value
    received: u16,
  },
  /// The operation `mlock` is not supported in your platform
  UnsupportedMlockPlatform,
  /// Unsupported operation
  UnsupportedOperation,
  /// Only appending is possible but overwritten is still viable through resetting.
  UriCanNotBeOverwritten,

  // Internal
  //
  #[doc = associated_element_doc!()]
  ArithmeticError(ArithmeticError),
  #[doc = associated_element_doc!()]
  ArrayStringError(ArrayStringError),
  #[doc = associated_element_doc!()]
  ArrayVectorError(ArrayVectorError),
  #[doc = associated_element_doc!()]
  AsciiError(AsciiError),
  #[cfg(feature = "asn1")]
  #[doc = associated_element_doc!()]
  Asn1Error(Asn1Error),
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
  DatabaseError(crate::database::DatabaseError),
  #[doc = associated_element_doc!()]
  ExecutorError(crate::executor::ExecutorError),
  #[doc = associated_element_doc!()]
  FixedStringError(FixedStringError),
  #[doc = associated_element_doc!()]
  FromRadix10Error(FromRadix10Error),
  #[doc = associated_element_doc!()]
  HexError(HexError),
  #[cfg(feature = "http")]
  #[doc = associated_element_doc!()]
  HttpError(crate::http::HttpError),
  /// Fatal HTTP/2 error
  #[cfg(feature = "http2")]
  Http2Error(crate::http2::Http2Error),
  /// Maybe Fatal HTTP/2 error
  #[cfg(feature = "http2")]
  Http2ErrorGoAway(crate::http2::Http2ErrorCode, crate::http2::Http2Error),
  /// Non-fatal HTTP/2 stream error
  #[cfg(feature = "http2")]
  Http2FlowControlError(crate::http2::Http2Error, u32),
  #[doc = associated_element_doc!()]
  NetError(crate::net::NetError),
  #[cfg(feature = "postgres")]
  #[doc = associated_element_doc!()]
  PostgresDbError(Box<crate::database::client::postgres::DbError>),
  #[cfg(feature = "postgres")]
  #[doc = associated_element_doc!()]
  PostgresError(crate::database::client::postgres::PostgresError),
  #[doc = associated_element_doc!()]
  QueueError(DequeueError),
  #[cfg(feature = "http")]
  #[doc = associated_element_doc!()]
  RouterError(crate::http::RouterError),
  #[cfg(feature = "schema-manager")]
  #[doc = associated_element_doc!()]
  SchemaManagerError(crate::database::schema_manager::SchemaManagerError),
  #[cfg(feature = "http2-server-framework")]
  #[doc = associated_element_doc!()]
  ServerFrameworkError(crate::http::http2_server_framework::Http2ServerFrameworkError),
  #[cfg(feature = "http-session")]
  #[doc = associated_element_doc!()]
  SessionError(crate::http::SessionError),
  #[cfg(feature = "tls")]
  #[doc = associated_element_doc!()]
  TlsError(crate::tls::TlsError),
  #[cfg(feature = "tls")]
  #[doc = associated_element_doc!()]
  TlsErrorFatal(crate::tls::TlsError, crate::tls::AlertDescription),
  #[doc = associated_element_doc!()]
  VectorError(VectorError),
  #[cfg(feature = "web-socket")]
  #[doc = associated_element_doc!()]
  WebSocketError(crate::web_socket::WebSocketError),
  #[cfg(feature = "x509")]
  #[doc = associated_element_doc!()]
  X509CvError(crate::x509::X509CvError),
  #[cfg(feature = "x509")]
  #[doc = associated_element_doc!()]
  X509Error(crate::x509::X509Error),
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

/// An error returned by the receiving part of a channel
#[derive(Clone, Copy, Debug)]
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
