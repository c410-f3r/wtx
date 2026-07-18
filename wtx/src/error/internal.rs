#[cfg(feature = "asn1")]
use crate::asn1::Asn1Error;
use crate::{
  Error,
  calendar::CalendarError,
  codec::{Base64Error, FromRadix10Error, HexError},
  collections::{
    ArrayStringError, ArrayVectorError, BlocksDequeError, DequeueError, FixedStringError,
    VectorError,
  },
  misc::{ArithmeticError, AsciiError},
};
#[allow(unused_imports, reason = "Depends on the selection of features")]
use alloc::boxed::Box;

impl From<ArithmeticError> for Error {
  #[inline]
  fn from(from: ArithmeticError) -> Self {
    Self::ArithmeticError(from)
  }
}

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

impl From<AsciiError> for Error {
  #[inline]
  fn from(from: AsciiError) -> Self {
    Self::AsciiError(from)
  }
}

#[cfg(feature = "asn1")]
impl From<Asn1Error> for Error {
  #[inline]
  fn from(from: Asn1Error) -> Self {
    Self::Asn1Error(from)
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

#[cfg(feature = "database")]
impl From<crate::database::DatabaseError> for Error {
  #[inline]
  fn from(from: crate::database::DatabaseError) -> Self {
    Self::DatabaseError(from)
  }
}

impl From<crate::executor::ExecutorError> for Error {
  #[inline]
  fn from(from: crate::executor::ExecutorError) -> Self {
    Self::ExecutorError(from)
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

#[cfg(feature = "http")]
impl From<crate::http::HttpError> for Error {
  #[inline]
  fn from(from: crate::http::HttpError) -> Self {
    Self::HttpError(from)
  }
}

impl From<crate::stream::BufStreamReaderError> for Error {
  #[inline]
  fn from(from: crate::stream::BufStreamReaderError) -> Self {
    Self::NetReadBufferError(from)
  }
}

#[cfg(feature = "postgres")]
impl From<crate::database::client::postgres::DbError> for Error {
  #[inline]
  fn from(from: crate::database::client::postgres::DbError) -> Self {
    Self::PostgresDbError(from.into())
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

#[cfg(feature = "http")]
impl From<crate::http::RouterError> for Error {
  #[inline]
  fn from(from: crate::http::RouterError) -> Self {
    Self::RouterError(from)
  }
}

#[cfg(feature = "schema-manager")]
impl From<crate::database::schema_manager::SchemaManagerError> for Error {
  #[inline]
  fn from(from: crate::database::schema_manager::SchemaManagerError) -> Self {
    Self::SchemaManagerError(from)
  }
}

#[cfg(feature = "http2-server-framework")]
impl From<crate::http::http2_server_framework::Http2ServerFrameworkError> for Error {
  #[inline]
  fn from(from: crate::http::http2_server_framework::Http2ServerFrameworkError) -> Self {
    Self::ServerFrameworkError(from)
  }
}

#[cfg(feature = "http-session")]
impl From<crate::http::SessionError> for Error {
  #[inline]
  fn from(from: crate::http::SessionError) -> Self {
    Self::SessionError(from)
  }
}

#[cfg(feature = "tls")]
impl From<crate::tls::TlsError> for Error {
  #[inline]
  fn from(from: crate::tls::TlsError) -> Self {
    Self::TlsError(from)
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
impl From<crate::x509::X509CvError> for Error {
  #[inline]
  fn from(from: crate::x509::X509CvError) -> Self {
    Self::X509CvError(from)
  }
}

#[cfg(feature = "x509")]
impl From<crate::x509::X509Error> for Error {
  #[inline]
  fn from(from: crate::x509::X509Error) -> Self {
    Self::X509Error(from)
  }
}
