use crate::misc::Lease;

macro_rules! create_statics {
  (
    $(
      $(#[$mac:meta])*
      $name:ident = $value:literal;
    )*
  ) => {
    impl HeaderNameStaticBytes {
      $(
        $(#[$mac])*
        #[doc = stringify!($name)]
        pub const $name: Self = Self::new($value);
      )*
    }
  };
}

create_statics! {
  ACCEPT = b"accept";
  ACCEPT_CHARSET = b"accept-charset";
  ACCEPT_ENCODING = b"accept-encoding";
  ACCEPT_LANGUAGE = b"accept-language";
  ACCEPT_RANGES = b"accept-ranges";
  ACCESS_CONTROL_ALLOW_CREDENTIALS = b"access-control-allow-credentials";
  ACCESS_CONTROL_ALLOW_HEADERS = b"access-control-allow-headers";
  ACCESS_CONTROL_ALLOW_METHODS = b"access-control-allow-methods";
  ACCESS_CONTROL_ALLOW_ORIGIN = b"access-control-allow-origin";
  ACCESS_CONTROL_EXPOSE_HEADERS = b"access-control-expose-headers";
  ACCESS_CONTROL_MAX_AGE = b"access-control-max-age";
  ACCESS_CONTROL_REQUEST_HEADERS = b"access-control-request-headers";
  ACCESS_CONTROL_REQUEST_METHOD = b"access-control-request-method";
  AGE = b"age";
  ALLOW = b"allow";
  AUTHORIZATION = b"authorization";
  CACHE_CONTROL = b"cache-control";
  CLEAR_SITE_DATA = b"clear-site-data";
  CONNECTION = b"connection";
  CONTENT_DISPOSITION = b"content-disposition";
  CONTENT_ENCODING = b"content-encoding";
  CONTENT_LANGUAGE = b"content-language";
  CONTENT_LENGTH = b"content-length";
  CONTENT_LOCATION = b"content-location";
  CONTENT_MD5 = b"content-md5";
  CONTENT_RANGE = b"content-range";
  CONTENT_TYPE = b"content-type";
  COOKIE = b"cookie";
  DATE = b"date";
  ETAG = b"etag";
  EXPECT = b"expect";
  EXPIRES = b"expires";
  FORWARDED = b"forwarded";
  FROM = b"from";
  HOST = b"host";
  IF_MATCH = b"if-match";
  IF_MODIFIED_SINCE = b"if-modified-since";
  IF_NONE_MATCH = b"if-none-match";
  IF_RANGE = b"if-range";
  IF_UNMODIFIED_SINCE = b"if-unmodified-since";
  KEEP_ALIVE = b"keep-alive";
  LAST_MODIFIED = b"last-modified";
  LINK = b"link";
  LOCATION = b"location";
  MAX_FORWARDS = b"max-forwards";
  ORIGIN = b"origin";
  PRAGMA = b"pragma";
  PROXY_AUTHENTICATE = b"proxy-authenticate";
  PROXY_AUTHORIZATION = b"proxy-authorization";
  PROXY_CONNECTION = b"proxy-connection";
  RANGE = b"range";
  REFERER = b"referer";
  REFRESH = b"refresh";
  RETRY_AFTER = b"retry-after";
  SERVER_TIMING = b"server-timing";
  SERVER = b"server";
  SET_COOKIE = b"set-cookie";
  SOURCE_MAP = b"sourcemap";
  STRICT_TRANSPORT_SECURITY = b"strict-transport-security";
  TE = b"te";
  TIMING_ALLOW_ORIGIN = b"timing-allow-origin";
  TRACEPARENT = b"traceparent";
  TRAILER = b"trailer";
  TRANSFER_ENCODING = b"transfer-encoding";
  UPGRADE = b"upgrade";
  USER_AGENT = b"user-agent";
  VARY = b"vary";
  VIA = b"via";
  WARNING = b"warning";
  WWW_AUTHENTICATE = b"www-authenticate";
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
