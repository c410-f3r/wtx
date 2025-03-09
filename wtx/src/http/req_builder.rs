use crate::{
  http::{Header, KnownHeaderName, Method, Mime, ReqResBuffer},
  misc::LeaseMut,
};
use core::fmt::Arguments;

/// Request builder
///
/// Provides shortcuts to manipulate requests through a fluent interface.
///
/// It is also possible to work directly with fields.
#[derive(Debug)]
pub struct ReqBuilder {
  /// Method
  pub method: Method,
  /// Buffer
  pub rrb: ReqResBuffer,
}

impl ReqBuilder {
  /// Constructor shortcut that has a default `GET` method
  #[inline]
  pub const fn get(rrb: ReqResBuffer) -> Self {
    Self { method: Method::Get, rrb }
  }

  /// Constructor shortcut that has a default `POST` method
  #[inline]
  pub const fn post(rrb: ReqResBuffer) -> Self {
    Self { method: Method::Get, rrb }
  }
}

impl ReqBuilder {
  /// Applies a header field in the form of `Authorization: Basic <credentials>` where
  /// `credentials` is the Base64 encoding of `id` and `pw` joined by a single colon `:`.
  #[inline]
  #[cfg(feature = "base64")]
  pub fn auth_basic(mut self, id: Arguments<'_>, pw: Arguments<'_>) -> crate::Result<Self> {
    use base64::{Engine, engine::general_purpose::STANDARD};
    use core::fmt::Write;
    let body_idx = self.rrb.body.len();
    let mut fun = || {
      self.rrb.uri._buffer(|buffer| {
        let uri_idx = buffer.len();
        buffer.write_fmt(format_args!("Basic {id}:{pw}"))?;
        let input = buffer.get(uri_idx..).unwrap_or_default();
        let _ = STANDARD.encode_slice(input, &mut self.rrb.body)?;
        Ok(())
      })?;
      let ReqResBuffer { body, headers, uri: _ } = self.rrb.lease_mut();
      headers.push_from_iter(Header::from_name_and_value(
        KnownHeaderName::Authorization.into(),
        [body.get(body_idx..).unwrap_or_default()],
      ))
    };
    if let Err(err) = fun() {
      self.rrb.body.truncate(body_idx);
      return Err(err);
    }
    Ok(self)
  }

  /// Applies a header field in the form of `Authorization: Bearer <token>`.
  #[inline]
  pub fn auth_bearer(mut self, token: Arguments<'_>) -> crate::Result<Self> {
    self.rrb.lease_mut().headers.push_from_fmt(Header::from_name_and_value(
      KnownHeaderName::Authorization.into(),
      format_args!("Bearer {token}"),
    ))?;
    Ok(self)
  }

  /// Injects a sequence of bytes into the internal buffer.
  ///
  /// No `content-type` header is applied in this method.
  #[inline]
  pub fn bytes(mut self, data: &[u8]) -> crate::Result<Self> {
    self.rrb.body.extend_from_copyable_slice(data)?;
    Ok(self)
  }

  /// Media type of the resource.
  #[inline]
  pub fn content_type(mut self, mime: Mime) -> crate::Result<Self> {
    self.rrb.lease_mut().headers.push_from_iter(Header::from_name_and_value(
      KnownHeaderName::ContentType.into(),
      [mime.as_str().as_bytes()],
    ))?;
    Ok(self)
  }

  /// Changes the method
  #[inline]
  pub fn method(mut self, method: Method) -> Self {
    self.method = method;
    self
  }

  /// Uses `serde_json` to inject a raw structure as JSON into the internal buffer.
  ///
  /// A `content-type` header of type `application/json` is also applied.
  #[inline]
  #[cfg(feature = "serde_json")]
  pub fn serde_json<T>(mut self, data: &T) -> crate::Result<Self>
  where
    T: serde::Serialize,
  {
    serde_json::to_writer(&mut self.rrb.body, data)?;
    self.content_type(Mime::ApplicationJson)
  }

  /// Uses `serde_urlencoded` to inject a raw structure as Percent-encoding into the internal
  /// buffer.
  ///
  /// A `content-type` header of type `application/x-www-form-urlencoded` is also applied.
  #[inline]
  #[cfg(feature = "serde_urlencoded")]
  pub fn serde_urlencoded<T>(mut self, data: &T) -> crate::Result<Self>
  where
    T: serde::Serialize,
  {
    self.rrb.body.extend_from_copyable_slice(serde_urlencoded::to_string(data)?.as_bytes())?;
    self.content_type(Mime::ApplicationXWwwFormUrlEncoded)
  }

  /// Injects a sequence of bytes into the internal buffer.
  ///
  /// A `content-type` header of type `text/plain` is also applied.
  #[inline]
  pub fn text(mut self, data: &[u8]) -> crate::Result<Self> {
    self.rrb.body.extend_from_copyable_slice(data)?;
    self.content_type(Mime::TextPlain)
  }

  /// Characteristic string that lets servers and network peers identify the application.
  #[inline]
  pub fn user_agent(mut self, value: &[u8]) -> crate::Result<Self> {
    self
      .rrb
      .lease_mut()
      .headers
      .push_from_iter(Header::from_name_and_value(KnownHeaderName::UserAgent.into(), [value]))?;
    Ok(self)
  }
}

impl Default for ReqBuilder {
  #[inline]
  fn default() -> Self {
    Self::get(ReqResBuffer::default())
  }
}
