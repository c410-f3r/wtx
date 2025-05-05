#![allow(non_upper_case_globals, reason = "macro parameters")]

macro_rules! create_statics {
  (
    $(
      $(#[$mac:meta])*
      $name:ident = $value:literal;
    )*
  ) => {
    /// A statically known set of header names
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum KnownHeaderName {
      $(
        $(#[$mac])*
        #[doc = stringify!($name)]
        $name,
      )*
    }

    impl From<KnownHeaderName> for &[u8] {
      #[inline]
      fn from(from: KnownHeaderName) -> Self {
        $( const $name: &[u8] = $value.as_bytes(); )*
        match from {
          $(
            KnownHeaderName::$name => $name,
          )*
        }
      }
    }

    impl From<KnownHeaderName> for &str {
      #[inline]
      fn from(from: KnownHeaderName) -> Self {
        match from {
          $(
            KnownHeaderName::$name => $value,
          )*
        }
      }
    }

    impl From<KnownHeaderName> for HeaderName<&str> {
      #[inline]
      fn from(from: KnownHeaderName) -> Self {
        Self(<&str>::from(from))
      }
    }

    impl<'bytes> TryFrom<&'bytes [u8]> for KnownHeaderName {
      type Error = HttpError;

      #[inline]
      fn try_from(from: &'bytes [u8]) -> Result<Self, Self::Error> {
        $( const $name: &[u8] = $value.as_bytes(); )*
        Ok(match from {
          $( $name => Self::$name, )*
          _ => return Err(HttpError::UnknownHeaderNameFromBytes {
            length: from.len()
          })
        })
      }
    }

    impl<S> TryFrom<HeaderName<S>> for KnownHeaderName
    where
      S: Lease<str>
    {
      type Error = HttpError;

      #[inline]
      fn try_from(from: HeaderName<S>) -> Result<Self, Self::Error> {
        KnownHeaderName::try_from(from.str().as_bytes())
      }
    }
  };
}

use crate::{
  http::HttpError,
  misc::{Lease, from_utf8_basic},
};
use core::str;

const TABLE: &[u8; 256] = &[
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, b'!', b'"', b'#', b'$', b'%', b'&', b'\'', 0, 0, b'*', b'+', 0, b'-', b'.', 0, b'0', b'1',
  b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, b'^', b'_', b'`', b'a', b'b', b'c',
  b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's',
  b't', b'u', b'v', b'w', b'x', b'y', b'z', 0, b'|', 0, b'~', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// HTTP header name
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HeaderName<S>(S);

impl<S> HeaderName<S>
where
  S: Lease<str>,
{
  /// New generic instance that doesn't verify `content`.
  #[inline]
  pub const fn new_unchecked(content: S) -> Self {
    Self(content)
  }

  /// Generic type content.
  #[inline]
  pub const fn content(&self) -> &S {
    &self.0
  }

  /// Generic type content in string form
  #[inline]
  pub fn str(&self) -> &str {
    self.0.lease()
  }

  fn check_header_name(content: &[u8]) -> crate::Result<&str> {
    if content.first().copied() == Some(b':') {
      return Ok(from_utf8_basic(content)?);
    }
    for idx in content {
      #[expect(
        clippy::indexing_slicing,
        reason = "An array of 256 elements will never panic with a 0..255 index"
      )]
      if TABLE[usize::from(*idx)] == 0 {
        return Err(HttpError::InvalidHttp2pContent.into());
      }
    }
    // SAFETY: `TABLE` is a subset of ASCII
    unsafe { Ok(str::from_utf8_unchecked(content)) }
  }
}

impl<'this> HeaderName<&'this str> {
  /// New instance from a set of bytes
  #[inline]
  pub fn from_bytes(content: &'this [u8]) -> crate::Result<Self> {
    Ok(HeaderName(Self::check_header_name(content)?))
  }
}

impl<'hn> From<HeaderName<&'hn str>> for HeaderName<&'hn [u8]> {
  #[inline]
  fn from(from: HeaderName<&'hn str>) -> Self {
    Self(from.0.as_bytes())
  }
}

create_statics! {
  Accept = "accept";
  AcceptCharset = "accept-charset";
  AcceptEncoding = "accept-encoding";
  AcceptLanguage = "accept-language";
  AcceptRanges = "accept-ranges";
  AccessControlAllowCredentials = "access-control-allow-credentials";
  AccessControlAllowHeaders = "access-control-allow-headers";
  AccessControlAllowMethods = "access-control-allow-methods";
  AccessControlAllowOrigin = "access-control-allow-origin";
  AccessControlExposeHeaders = "access-control-expose-headers";
  AccessControlMaxAge = "access-control-max-age";
  AccessControlRequestHeaders = "access-control-request-headers";
  AccessControlRequestMethod = "access-control-request-method";
  Age = "age";
  Allow = "allow";
  Authorization = "authorization";
  CacheControl = "cache-control";
  ClearSiteData = "clear-site-data";
  Connection = "connection";
  ContentDisposition = "content-disposition";
  ContentEncoding = "content-encoding";
  ContentLanguage = "content-language";
  ContentLength = "content-length";
  ContentLocation = "content-location";
  ContentMd5 = "content-md5";
  ContentRange = "content-range";
  ContentType = "content-type";
  Cookie = "cookie";
  Date = "date";
  Etag = "etag";
  Expect = "expect";
  Expires = "expires";
  Forwarded = "forwarded";
  From = "from";
  Host = "host";
  IfMatch = "if-match";
  IfModifiedSince = "if-modified-since";
  IfNoneMatch = "if-none-match";
  IfRange = "if-range";
  IfUnmodifiedSince = "if-unmodified-since";
  KeepAlive = "keep-alive";
  LastModified = "last-modified";
  Link = "link";
  Location = "location";
  MaxForwards = "max-forwards";
  Origin = "origin";
  Pragma = "pragma";
  ProxyAuthenticate = "proxy-authenticate";
  ProxyAuthorization = "proxy-authorization";
  ProxyConnection = "proxy-connection";
  Range = "range";
  Referer = "referer";
  Refresh = "refresh";
  RetryAfter = "retry-after";
  SecWebsocketAccept = "sec-websocket-accept";
  SecWebsocketExtensions = "sec-websocket-extensions";
  SecWebsocketKey = "sec-websocket-key";
  SecWebsocketVersion = "sec-websocket-version";
  Server = "server";
  ServerTiming = "server-timing";
  SetCookie = "set-cookie";
  SourceMap = "sourcemap";
  StrictTransportSecurity = "strict-transport-security";
  Te = "te";
  TimingAllowOrigin = "timing-allow-origin";
  Traceparent = "traceparent";
  Trailer = "trailer";
  TransferEncoding = "transfer-encoding";
  Upgrade = "upgrade";
  UserAgent = "user-agent";
  Vary = "vary";
  Via = "via";
  Warning = "warning";
  WwwAuthenticate = "www-authenticate";
  XCsrfToken = "x-csrf-token";
}
