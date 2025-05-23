create_enum! {
  /// HTTP status codes.
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub enum StatusCode<u16> {
      /// 100 Continue
      Continue = (100),
      /// 101 Switching Protocols
      SwitchingProtocols = (101),
      /// 103 Early Hints
      EarlyHints = (103),
      //
      //
      /// 200 Ok
      Ok = (200),
      /// 201 Created
      Created = (201),
      /// 202 Accepted
      Accepted = (202),
      /// 203 Non Authoritative Information
      NonAuthoritativeInformation = (203),
      /// 204 No Content
      NoContent = (204),
      /// 205 Reset Content
      ResetContent = (205),
      /// 206 Partial Content
      PartialContent = (206),
      /// 207 Multi-Status
      MultiStatus = (207),
      /// 226 Im Used
      ImUsed = (226),
      //
      //
      /// 300 Multiple Choice
      MultipleChoice = (300),
      /// 301 Moved Permanently
      MovedPermanently = (301),
      /// 302 Found
      Found = (302),
      /// 303 See Other
      SeeOther = (303),
      /// 304 Not Modified
      NotModified = (304),
      /// 307 Temporary Redirect
      TemporaryRedirect = (307),
      /// 308 Permanent Redirect
      PermanentRedirect = (308),
      //
      //
      /// 400 Bad Request
      BadRequest = (400),
      /// 401 Unauthorized
      Unauthorized = (401),
      /// 402 Payment Required
      PaymentRequired = (402),
      /// 403 Forbidden
      Forbidden = (403),
      /// 404 Not Found
      NotFound = (404),
      /// 405 Method Not Allowed
      MethodNotAllowed = (405),
      /// 406 Not Acceptable
      NotAcceptable = (406),
      /// 407 Proxy Authentication Required
      ProxyAuthenticationRequired = (407),
      /// 408 Request Timeout
      RequestTimeout = (408),
      /// 409 Conflict
      Conflict = (409),
      /// 410 Gone
      Gone = (410),
      /// 411 Length Required
      LengthRequired = (411),
      /// 412 Precondition Failed
      PreconditionFailed = (412),
      /// 413 Payload Too Large
      PayloadTooLarge = (413),
      /// 414 URI Too Long
      UriTooLong = (414),
      /// 415 Unsupported Media Type
      UnsupportedMediaType = (415),
      /// 416 Requested Range Not Satisfiable
      RequestedRangeNotSatisfiable = (416),
      /// 417 Expectation Failed
      ExpectationFailed = (417),
      /// 418 I'm a teapot
      ImATeapot = (418),
      /// 421 Misdirected Request
      MisdirectedRequest = (421),
      /// 422 Unprocessable Entity
      UnprocessableEntity = (422),
      /// 423 Locked
      Locked = (423),
      /// 424 Failed Dependency
      FailedDependency = (424),
      /// 425 Too Early
      TooEarly = (425),
      /// 426 Upgrade Required
      UpgradeRequired = (426),
      /// 428 Precondition Required
      PreconditionRequired = (428),
      /// 429 Too Many Requests
      TooManyRequests = (429),
      /// 431 Request Header Fields Too Large
      RequestHeaderFieldsTooLarge = (431),
      /// 451 Unavailable For Legal Reasons
      UnavailableForLegalReasons = (451),
      //
      //
      /// 500 Internal Server Error
      InternalServerError = (500),
      /// 501 Not Implemented
      NotImplemented = (501),
      /// 502 Bad Gateway
      BadGateway = (502),
      /// 503 Service Unavailable
      ServiceUnavailable = (503),
      /// 504 Gateway Timeout
      GatewayTimeout = (504),
      /// 505 HTTP Version Not Supported
      HttpVersionNotSupported = (505),
      /// 506 Variant Also Negotiates
      VariantAlsoNegotiates = (506),
      /// 507 Insufficient Storage
      InsufficientStorage = (507),
      /// 508 Loop Detected
      LoopDetected = (508),
      /// 510 Not Extended
      NotExtended = (510),
      /// 511 Network Authentication Required
      NetworkAuthenticationRequired = (511),
  }
}

#[cfg(feature = "http-server-framework")]
mod http_server_framework {
  use crate::http::{ReqResBuffer, Request, StatusCode, server_framework::ResFinalizer};

  impl<E> ResFinalizer<E> for StatusCode
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn finalize_response(self, req: &mut Request<ReqResBuffer>) -> Result<StatusCode, E> {
      req.rrd.clear();
      Ok(self)
    }
  }
}
