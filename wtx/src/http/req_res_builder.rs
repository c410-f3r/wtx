mod req_builder;
mod res_builder;

use crate::{
  collection::Vector,
  http::{Header, KnownHeaderName, Mime, ReqResBuffer, ReqResDataMut},
  misc::LeaseMut,
};
use core::fmt::Arguments;
pub use req_builder::ReqBuilder;
pub use res_builder::ResBuilder;

/// Request/Response Builder
#[derive(Debug)]
pub struct ReqResBuilder<RRD> {
  /// Request/Response Data
  pub rrd: RRD,
}

impl<RRD> ReqResBuilder<RRD> {
  /// Constructor shortcut
  #[inline]
  pub const fn new(rrd: RRD) -> Self {
    Self { rrd }
  }
}

impl ReqResBuilder<ReqResBuffer> {
  /// Applies a header field in the form of `Authorization: Basic <credentials>` where
  /// `credentials` is the Base64 encoding of `id` and `pw` joined by a single colon `:`.
  #[cfg(feature = "base64")]
  #[inline]
  pub fn auth_basic(&mut self, id: Arguments<'_>, pw: Arguments<'_>) -> crate::Result<&mut Self> {
    use base64::{Engine, engine::general_purpose::STANDARD};
    use core::fmt::Write;
    let ReqResBuffer { body, headers, uri } = self.rrd.lease_mut();
    let body_idx = body.len();
    let mut fun = || {
      uri.buffer(|buffer| {
        let uri_idx = buffer.len();
        buffer.write_fmt(format_args!("Basic {id}:{pw}"))?;
        let input = buffer.get(uri_idx..).unwrap_or_default();
        let _ = STANDARD.encode_slice(input, body)?;
        Ok(())
      })?;
      headers.push_from_iter(Header::from_name_and_value(
        KnownHeaderName::Authorization.into(),
        // SAFETY: Everything after `body_idx` is UTF-8
        [unsafe { core::str::from_utf8_unchecked(body.get(body_idx..).unwrap_or_default()) }],
      ))
    };
    if let Err(err) = fun() {
      body.truncate(body_idx);
      return Err(err);
    }
    Ok(self)
  }
}

impl<RRD> ReqResBuilder<RRD>
where
  RRD: ReqResDataMut,
{
  /// Which formats the request should receive.
  #[inline]
  pub fn accept(&mut self, value: Arguments<'_>) -> crate::Result<&mut Self> {
    self
      .rrd
      .headers_mut()
      .push_from_fmt(Header::from_name_and_value(KnownHeaderName::Accept.into(), value))?;
    Ok(self)
  }

  /// Applies a header field in the form of `Authorization: Bearer <token>`.
  #[inline]
  pub fn auth_bearer(&mut self, token: Arguments<'_>) -> crate::Result<&mut Self> {
    self.rrd.headers_mut().push_from_fmt(Header::from_name_and_value(
      KnownHeaderName::Authorization.into(),
      format_args!("Bearer {token}"),
    ))?;
    Ok(self)
  }

  /// Media type of the resource.
  #[inline]
  pub fn content_type(&mut self, mime: Mime) -> crate::Result<&mut Self> {
    self.rrd.headers_mut().push_from_iter(Header::from_name_and_value(
      KnownHeaderName::ContentType.into(),
      [mime.as_str()],
    ))?;
    Ok(self)
  }

  /// Adds a header with a value composed by an iterator.
  #[inline]
  pub fn header_iter<'kv, I>(&mut self, name: &'kv str, value: I) -> crate::Result<&mut Self>
  where
    I: IntoIterator<Item = &'kv str>,
    I::IntoIter: Clone,
  {
    self.rrd.headers_mut().push_from_iter(Header::from_name_and_value(name, value))?;
    Ok(self)
  }

  /// Adds a header with a value composed by [`Arguments`].
  #[inline]
  pub fn header_fmt(&mut self, name: &str, value: Arguments<'_>) -> crate::Result<&mut Self> {
    self.rrd.headers_mut().push_from_fmt(Header::from_name_and_value(name, value))?;
    Ok(self)
  }

  /// The host and port number of the server to which the request is being sent.
  #[inline]
  pub fn host(&mut self, value: Arguments<'_>) -> crate::Result<&mut Self> {
    self
      .rrd
      .headers_mut()
      .push_from_fmt(Header::from_name_and_value(KnownHeaderName::Host.into(), value))?;
    Ok(self)
  }

  /// Characteristic string that lets servers and network peers identify the application.
  #[inline]
  pub fn user_agent(&mut self, value: &str) -> crate::Result<&mut Self> {
    self
      .rrd
      .headers_mut()
      .push_from_iter(Header::from_name_and_value(KnownHeaderName::UserAgent.into(), [value]))?;
    Ok(self)
  }
}

impl<RRD> ReqResBuilder<RRD>
where
  RRD: ReqResDataMut,
  RRD::Body: LeaseMut<Vector<u8>>,
{
  /// Injects a sequence of bytes into the internal buffer.
  ///
  /// No `content-type` header is applied in this method.
  #[inline]
  pub fn bytes(&mut self, data: &[u8]) -> crate::Result<&mut Self> {
    self.rrd.body_mut().lease_mut().extend_from_copyable_slice(data)?;
    Ok(self)
  }

  /// Uses `serde_json` to inject a raw structure as JSON into the internal buffer.
  ///
  /// A `content-type` header of type `application/json` is also applied.
  #[cfg(feature = "serde_json")]
  #[inline]
  pub fn serde_json<T>(&mut self, data: &T) -> crate::Result<&mut Self>
  where
    T: serde::Serialize,
  {
    serde_json::to_writer(self.rrd.body_mut().lease_mut(), data)?;
    self.content_type(Mime::ApplicationJson)
  }

  /// Uses `serde_urlencoded` to inject a raw structure as Percent-encoding into the internal
  /// buffer.
  ///
  /// A `content-type` header of type `application/x-www-form-urlencoded` is also applied.
  #[cfg(feature = "serde_urlencoded")]
  #[inline]
  pub fn serde_urlencoded<T>(&mut self, data: &T) -> crate::Result<&mut Self>
  where
    T: serde::Serialize,
  {
    self
      .rrd
      .body_mut()
      .lease_mut()
      .extend_from_copyable_slice(serde_urlencoded::to_string(data)?.as_bytes())?;
    self.content_type(Mime::ApplicationXWwwFormUrlEncoded)
  }

  /// Injects a sequence of bytes into the internal buffer.
  ///
  /// A `content-type` header of type `text/plain` is also applied.
  #[inline]
  pub fn text(&mut self, data: &[u8]) -> crate::Result<&mut Self> {
    self.rrd.body_mut().lease_mut().extend_from_copyable_slice(data)?;
    self.content_type(Mime::TextPlain)
  }
}
