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
    Self::Arithmetic(from)
  }
}

impl From<ArrayStringError> for Error {
  #[inline]
  fn from(from: ArrayStringError) -> Self {
    Self::ArrayString(from)
  }
}

impl From<ArrayVectorError> for Error {
  #[inline]
  fn from(from: ArrayVectorError) -> Self {
    Self::ArrayVector(from)
  }
}

impl From<AsciiError> for Error {
  #[inline]
  fn from(from: AsciiError) -> Self {
    Self::Ascii(from)
  }
}

#[cfg(feature = "asn1")]
impl From<Asn1Error> for Error {
  #[inline]
  fn from(from: Asn1Error) -> Self {
    Self::Asn1(from)
  }
}

impl From<Base64Error> for Error {
  #[inline]
  fn from(from: Base64Error) -> Self {
    Self::Base64(from)
  }
}

impl From<BlocksDequeError> for Error {
  #[inline]
  fn from(from: BlocksDequeError) -> Self {
    Self::BlocksQueue(from)
  }
}

impl From<CalendarError> for Error {
  #[inline]
  fn from(from: CalendarError) -> Self {
    Self::Calendar(from)
  }
}

#[cfg(feature = "client-api-framework")]
impl From<crate::client_api_framework::ClientApiFrameworkError> for Error {
  #[inline]
  fn from(from: crate::client_api_framework::ClientApiFrameworkError) -> Self {
    Self::ClientApiFramework(from)
  }
}

impl From<crate::codec::CodecError> for Error {
  #[inline]
  fn from(from: crate::codec::CodecError) -> Self {
    Self::Codec(from.into())
  }
}

#[cfg(feature = "http-cookie")]
impl From<crate::http::cookie::CookieError> for Error {
  #[inline]
  #[track_caller]
  fn from(from: crate::http::cookie::CookieError) -> Self {
    Self::Cookie(from)
  }
}

#[cfg(feature = "crypto")]
impl From<crate::crypto::CryptoError> for Error {
  #[inline]
  #[track_caller]
  fn from(from: crate::crypto::CryptoError) -> Self {
    Self::Crypto(from)
  }
}

impl From<crate::collections::CsrError> for Error {
  #[inline]
  #[track_caller]
  fn from(from: crate::collections::CsrError) -> Self {
    Self::Csr(from)
  }
}

#[cfg(feature = "database")]
impl From<crate::database::DatabaseError> for Error {
  #[inline]
  fn from(from: crate::database::DatabaseError) -> Self {
    Self::Database(from)
  }
}

impl From<crate::executor::ExecutorError> for Error {
  #[inline]
  fn from(from: crate::executor::ExecutorError) -> Self {
    Self::Executor(from)
  }
}

impl From<FixedStringError> for Error {
  #[inline]
  fn from(from: FixedStringError) -> Self {
    Self::FixedString(from)
  }
}

impl From<FromRadix10Error> for Error {
  #[inline]
  fn from(from: FromRadix10Error) -> Self {
    Self::FromRadix10(from)
  }
}

impl From<HexError> for Error {
  #[inline]
  fn from(from: HexError) -> Self {
    Self::Hex(from)
  }
}

#[cfg(feature = "http")]
impl From<crate::http::HttpError> for Error {
  #[inline]
  fn from(from: crate::http::HttpError) -> Self {
    Self::Http(from)
  }
}

impl From<crate::net::NetError> for Error {
  #[inline]
  fn from(from: crate::net::NetError) -> Self {
    Self::Net(from)
  }
}

#[cfg(feature = "postgres")]
impl From<crate::database::client::postgres::DbError> for Error {
  #[inline]
  fn from(from: crate::database::client::postgres::DbError) -> Self {
    Self::PostgresDb(from.into())
  }
}

#[cfg(feature = "postgres")]
impl From<crate::database::client::postgres::PostgresError> for Error {
  #[inline]
  fn from(from: crate::database::client::postgres::PostgresError) -> Self {
    Self::Postgres(from)
  }
}

impl From<DequeueError> for Error {
  #[inline]
  fn from(from: DequeueError) -> Self {
    Self::Queue(from)
  }
}

#[cfg(feature = "http")]
impl From<crate::http::RouterError> for Error {
  #[inline]
  fn from(from: crate::http::RouterError) -> Self {
    Self::Router(from)
  }
}

#[cfg(feature = "schema-manager")]
impl From<crate::database::schema_manager::SchemaManagerError> for Error {
  #[inline]
  fn from(from: crate::database::schema_manager::SchemaManagerError) -> Self {
    Self::SchemaManager(from)
  }
}

#[cfg(feature = "http2-server-framework")]
impl From<crate::http::http2_server_framework::Http2ServerFrameworkError> for Error {
  #[inline]
  fn from(from: crate::http::http2_server_framework::Http2ServerFrameworkError) -> Self {
    Self::ServerFramework(from)
  }
}

#[cfg(feature = "http-session")]
impl From<crate::http::SessionError> for Error {
  #[inline]
  fn from(from: crate::http::SessionError) -> Self {
    Self::Session(from)
  }
}

#[cfg(feature = "tls")]
impl From<crate::tls::TlsError> for Error {
  #[inline]
  fn from(from: crate::tls::TlsError) -> Self {
    Self::Tls(from)
  }
}

impl From<VectorError> for Error {
  #[inline]
  fn from(from: VectorError) -> Self {
    Self::Vector(from)
  }
}

#[cfg(feature = "web-socket")]
impl From<crate::web_socket::WebSocketError> for Error {
  #[inline]
  fn from(from: crate::web_socket::WebSocketError) -> Self {
    Self::WebSocket(from)
  }
}

#[cfg(feature = "x509")]
impl From<crate::x509::X509CvError> for Error {
  #[inline]
  fn from(from: crate::x509::X509CvError) -> Self {
    Self::X509Cv(from)
  }
}

#[cfg(feature = "x509")]
impl From<crate::x509::X509Error> for Error {
  #[inline]
  fn from(from: crate::x509::X509Error) -> Self {
    Self::X509(from)
  }
}
