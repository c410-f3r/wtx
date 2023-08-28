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

    /// Missing Header
    MissingHeader {
        /// See [ExpectedHeader].
        expected: ExpectedHeader,
    },
    /// Url does not contain a host.
    MissingHost,

    /// HTTP version does not match the expected method.
    UnexpectedHttpMethod,
    /// HTTP version does not match the expected value.
    UnexpectedHttpVersion,
    /// Unexpected end of file when reading.
    UnexpectedEOF,

    /// The system does not process HTTP messages greater than 2048 bytes.
    VeryLargeHttp,

    // External
    //
    /// See [glommio::GlommioError].
    #[cfg(all(feature = "glommio", feature = "hyper"))]
    Glommio(std::sync::Mutex<glommio::GlommioError<()>>),
    /// See [glommio::GlommioError].
    #[cfg(all(feature = "glommio", not(feature = "hyper")))]
    Glommio(Box<glommio::GlommioError<()>>),
    #[cfg(feature = "http")]
    /// See [hyper::Error]
    HttpError(http::Error),
    /// See [http::header::InvalidHeaderName]
    #[cfg(feature = "http")]
    HttpInvalidHeaderName(http::header::InvalidHeaderName),
    /// See [http::header::InvalidHeaderValue]
    #[cfg(feature = "http")]
    HttpInvalidHeaderValue(http::header::InvalidHeaderValue),
    /// See [http::status::InvalidStatusCode]
    #[cfg(feature = "http")]
    HttpInvalidStatusCode(http::status::InvalidStatusCode),
    #[cfg(feature = "web-socket-handshake")]
    /// See [httparse::Error].
    HttpParse(httparse::Error),
    #[cfg(feature = "hyper")]
    /// See [hyper::Error]
    HyperError(hyper::Error),
    #[cfg(feature = "std")]
    /// See [std::io::Error]
    IoError(std::io::Error),
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

#[cfg(feature = "glommio")]
impl From<glommio::GlommioError<()>> for Error {
    #[inline]
    fn from(from: glommio::GlommioError<()>) -> Self {
        Self::Glommio(from.into())
    }
}

#[cfg(feature = "hyper")]
impl From<hyper::Error> for Error {
    #[inline]
    fn from(from: hyper::Error) -> Self {
        Self::HyperError(from)
    }
}

#[cfg(feature = "http")]
impl From<http::Error> for Error {
    #[inline]
    fn from(from: http::Error) -> Self {
        Self::HttpError(from)
    }
}

#[cfg(feature = "http")]
impl From<http::header::InvalidHeaderName> for Error {
    #[inline]
    fn from(from: http::header::InvalidHeaderName) -> Self {
        Self::HttpInvalidHeaderName(from)
    }
}

#[cfg(feature = "http")]
impl From<http::header::InvalidHeaderValue> for Error {
    #[inline]
    fn from(from: http::header::InvalidHeaderValue) -> Self {
        Self::HttpInvalidHeaderValue(from)
    }
}

#[cfg(feature = "http")]
impl From<http::status::InvalidStatusCode> for Error {
    #[inline]
    fn from(from: http::status::InvalidStatusCode) -> Self {
        Self::HttpInvalidStatusCode(from)
    }
}

#[cfg(feature = "web-socket-handshake")]
impl From<httparse::Error> for Error {
    #[inline]
    fn from(from: httparse::Error) -> Self {
        Self::HttpParse(from)
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

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    #[inline]
    fn from(from: std::io::Error) -> Self {
        Self::IoError(from)
    }
}
