#![allow(non_upper_case_globals)]

use crate::misc::Lease;

macro_rules! create_statics {
  (
    $(
      $(#[$mac:meta])*
      $name:ident = $value:literal;
    )*
  ) => {
    /// A statically known set of header names
    #[derive(Debug)]
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

    impl From<KnownHeaderName> for HeaderNameStaticBytes {
      #[inline]
      fn from(from: KnownHeaderName) -> Self {
        HeaderNameStaticBytes::new(<&[u8]>::from(from))
      }
    }

    impl From<KnownHeaderName> for HeaderNameStaticStr {
      #[inline]
      fn from(from: KnownHeaderName) -> Self {
        HeaderNameStaticStr::new(<&str>::from(from))
      }
    }

    impl<'bytes> TryFrom<&'bytes [u8]> for KnownHeaderName {
      type Error = crate::Error;

      #[inline]
      fn try_from(from: &'bytes [u8]) -> Result<Self, Self::Error> {
        $( const $name: &[u8] = $value.as_bytes(); )*
        Ok(match from {
          $( $name => Self::$name, )*
          _ => return Err(crate::Error::HTTP_UnknownHeaderNameFromBytes {
            length: from.len()
          })
        })
      }
    }
  };
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
  SecWebsocketVersion = "sec-websocket-version";
  SecWebsocketKey = "sec-websocket-key";
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
}

/// [HeaderName] composed by static bytes.
pub type HeaderNameStaticBytes = HeaderName<&'static [u8]>;
/// [HeaderName] composed by a static string.
pub type HeaderNameStaticStr = HeaderName<&'static str>;

/// HTTP header name
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HeaderName<S>(S);

impl<S> HeaderName<S>
where
  S: Lease<[u8]>,
{
  /// Instance from a generic type content.
  #[inline]
  pub const fn new(content: S) -> Self {
    Self(content)
  }

  /// Generic type content in bytes form
  #[inline]
  pub fn bytes(&self) -> &[u8] {
    self.0.lease()
  }

  /// Generic type content.
  #[inline]
  pub const fn content(&self) -> &S {
    &self.0
  }
}

impl<'hn> From<HeaderName<&'hn str>> for HeaderName<&'hn [u8]> {
  #[inline]
  fn from(from: HeaderName<&'hn str>) -> Self {
    Self::new(from.0.as_bytes())
  }
}
