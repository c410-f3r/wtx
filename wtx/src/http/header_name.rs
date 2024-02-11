macro_rules! create_statics {
  (
    $(
      $(#[$mac:meta])*
      $name:ident = $value:literal;
    )*
  ) => {
    $(
      $(#[$mac])*
      pub const $name: HeaderNameStaticStr = HeaderNameStaticStr::new($value);
    )*

    impl<'hn> TryFrom<&'hn [u8]> for HeaderName<&'hn [u8]> {
      type Error = crate::Error;

      #[inline]
      fn try_from(from: &'hn [u8]) -> crate::Result<Self> {
        Ok(match from {
          $(
            elem if elem == $value.as_bytes() => $name.into(),
          )*
          _ => Self(from)
        })
      }
    }

    impl<'hn> TryFrom<&'hn str> for HeaderName<&'hn str> {
      type Error = crate::Error;

      #[inline]
      fn try_from(from: &'hn str) -> crate::Result<Self> {
        Ok(match from {
          $(
            $value => $name,
          )*
          _ => Self(from)
        })
      }
    }

    impl<'hn> From<HeaderName<&'hn str>> for HeaderName<&'hn [u8]> {
      #[inline]
      fn from(from: HeaderName<&'hn str>) -> Self {
        Self::new(from.0.as_bytes())
      }
    }
  };
}

create_statics! {
  /// accept
  ACCEPT = "accept";
  /// accept-charset
  ACCEPT_CHARSET = "accept-charset";
  /// accept-encoding
  ACCEPT_ENCODING = "accept-encoding";
  /// accept-language
  ACCEPT_LANGUAGE = "accept-language";
  /// accept-ranges
  ACCEPT_RANGES = "accept-ranges";
  /// access-control-allow-credentials
  ACCESS_CONTROL_ALLOW_CREDENTIALS = "access-control-allow-credentials";
  /// access-control-allow-headers
  ACCESS_CONTROL_ALLOW_HEADERS = "access-control-allow-headers";
  /// access-control-allow-methods
  ACCESS_CONTROL_ALLOW_METHODS = "access-control-allow-methods";
  /// access-control-allow-origin
  ACCESS_CONTROL_ALLOW_ORIGIN = "access-control-allow-origin";
  /// access-control-expose-headers
  ACCESS_CONTROL_EXPOSE_HEADERS = "access-control-expose-headers";
  /// access-control-max-age
  ACCESS_CONTROL_MAX_AGE = "access-control-max-age";
  /// access-control-request-headers
  ACCESS_CONTROL_REQUEST_HEADERS = "access-control-request-headers";
  /// access-control-request-method
  ACCESS_CONTROL_REQUEST_METHOD = "access-control-request-method";
  /// age
  AGE = "age";
  /// allow
  ALLOW = "allow";
  /// authorization
  AUTHORIZATION = "authorization";
  /// cache-control
  CACHE_CONTROL = "cache-control";
  /// clear-site-data
  CLEAR_SITE_DATA = "clear-site-data";
  /// Connection
  CONNECTION = "connection";
  /// content-disposition
  CONTENT_DISPOSITION = "content-disposition";
  /// content-encoding
  CONTENT_ENCODING = "content-encoding";
  /// content-language
  CONTENT_LANGUAGE = "content-language";
  /// content-length
  CONTENT_LENGTH = "content-length";
  /// content-location
  CONTENT_LOCATION = "content-location";
  /// content-md5
  CONTENT_MD5 = "content-md5";
  /// content-range
  CONTENT_RANGE = "content-range";
  /// content-type
  CONTENT_TYPE = "content-type";
  /// cookie
  COOKIE = "cookie";
  /// date
  DATE = "date";
  /// eTag
  ETAG = "etag";
  /// expect
  EXPECT = "expect";
  /// expires
  EXPIRES = "expires";
  /// forwarded
  FORWARDED = "forwarded";
  /// from
  FROM = "from";
  /// host
  HOST = "host";
  /// if-match
  IF_MATCH = "if-match";
  /// if-modified-since
  IF_MODIFIED_SINCE = "if-modified-since";
  /// if-none-match
  IF_NONE_MATCH = "if-none-match";
  /// if-range
  IF_RANGE = "if-range";
  /// if-unmodified-since
  IF_UNMODIFIED_SINCE = "if-unmodified-since";
  /// last-modified
  LAST_MODIFIED = "last-modified";
  /// link
  LINK = "link";
  /// location
  LOCATION = "location";
  /// max-forwards
  MAX_FORWARDS = "max-forwards";
  /// origin
  ORIGIN = "origin";
  /// pragma
  PRAGMA = "pragma";
  /// proxy-authenticate
  PROXY_AUTHENTICATE = "proxy-authenticate";
  /// proxy-authorization
  PROXY_AUTHORIZATION = "proxy-authorization";
  /// proxy-connection
  PROXY_CONNECTION = "proxy-connection";
  /// range
  RANGE = "range";
  /// referer
  REFERER = "referer";
  /// refresh
  REFRESH = "refresh";
  /// retry-after
  RETRY_AFTER = "retry-after";
  /// server
  SERVER_TIMING = "server-timing";
  /// server
  SERVER = "server";
  /// set-cookie
  SET_COOKIE = "set-cookie";
  /// sourcemap
  SOURCE_MAP = "sourcemap";
  /// strict-transport-security
  STRICT_TRANSPORT_SECURITY = "strict-transport-security";
  /// te
  TE = "te";
  /// timing-allow-origin
  TIMING_ALLOW_ORIGIN = "timing-allow-origin";
  /// traceparent
  TRACEPARENT = "traceparent";
  /// trailer
  TRAILER = "trailer";
  /// transfer-encoding
  TRANSFER_ENCODING = "transfer-encoding";
  /// upgrade
  UPGRADE = "upgrade";
  /// user-Agent
  USER_AGENT = "user-agent";
  /// vary
  VARY = "vary";
  /// via
  VIA = "via";
  /// warning
  WARNING = "warning";
  /// www-authenticate
  WWW_AUTHENTICATE = "www-authenticate";
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
  S: AsRef<[u8]>,
{
  /// Instance from a generic type content.
  #[inline]
  pub const fn new(content: S) -> Self {
    Self(content)
  }

  /// Generic type content in bytes form
  #[inline]
  pub fn bytes(&self) -> &[u8] {
    self.0.as_ref()
  }

  /// Generic type content.
  #[inline]
  pub const fn content(&self) -> &S {
    &self.0
  }
}
