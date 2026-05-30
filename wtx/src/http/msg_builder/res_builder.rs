use crate::http::{KnownHeaderName, MsgBuilder, MsgBuilderInput, MsgDataMut, Response, StatusCode};
use core::ops::{Deref, DerefMut};

/// Response builder
///
/// Provides shortcuts to manipulate responses through a fluent interface.
#[derive(Debug)]
pub struct ResBuilder<MD> {
  /// Generic builder
  msg_builder: MsgBuilder<MD>,
  /// Status code
  status_code: StatusCode,
}

impl<MD> ResBuilder<MD> {
  /// A new empty response with a 400 Bad Request status code.
  #[inline]
  pub const fn bad_request(msg_data: MD) -> Self {
    Self { msg_builder: MsgBuilder::new(msg_data), status_code: StatusCode::BadRequest }
  }

  /// A new empty response with a 201 Created status code.
  #[inline]
  pub const fn created(msg_data: MD) -> Self {
    Self { msg_builder: MsgBuilder::new(msg_data), status_code: StatusCode::Created }
  }

  /// A new empty response with a 500 Internal Server Error status code.
  #[inline]
  pub const fn internal_server_error(msg_data: MD) -> Self {
    Self { msg_builder: MsgBuilder::new(msg_data), status_code: StatusCode::InternalServerError }
  }

  /// A new empty response with a 404 Not Found status code.
  #[inline]
  pub const fn not_found(msg_data: MD) -> Self {
    Self { msg_builder: MsgBuilder::new(msg_data), status_code: StatusCode::NotFound }
  }

  /// A new empty response with a 200 OK status code.
  #[inline]
  pub const fn ok(msg_data: MD) -> Self {
    Self { msg_builder: MsgBuilder::new(msg_data), status_code: StatusCode::Ok }
  }

  /// Shortcut that converts this instance into a [`Response`].
  #[inline]
  pub fn into_response(self) -> Response<MD> {
    Response::new(self.msg_builder.msg_data, self.status_code)
  }

  /// Changes the status code
  #[inline]
  pub const fn status_code(&mut self, status_code: StatusCode) -> &mut Self {
    self.status_code = status_code;
    self
  }
}

impl<MD> ResBuilder<MD>
where
  MD: MsgDataMut,
{
  /// Sets the `Location` header, typically used with 3xx redirect status codes.
  #[inline]
  pub fn location<'left, 'right, I>(&mut self, uri: I) -> crate::Result<&mut Self>
  where
    I: Into<MsgBuilderInput<'left, 'right>>,
  {
    self.msg_builder.push_header(KnownHeaderName::Location.into(), uri)?;
    Ok(self)
  }

  /// Sets the `Set-Cookie` header.
  #[inline]
  pub fn set_cookie<'left, 'right, I>(&mut self, cookie: I) -> crate::Result<&mut Self>
  where
    I: Into<MsgBuilderInput<'left, 'right>>,
  {
    self.msg_builder.push_header(KnownHeaderName::SetCookie.into(), cookie)?;
    Ok(self)
  }

  /// Sets the `Cache-Control` header.
  #[inline]
  pub fn cache_control<'left, 'right, I>(&mut self, policy: I) -> crate::Result<&mut Self>
  where
    I: Into<MsgBuilderInput<'left, 'right>>,
  {
    self.msg_builder.push_header(KnownHeaderName::CacheControl.into(), policy)?;
    Ok(self)
  }
}

impl<MD> Default for ResBuilder<MD>
where
  MD: Default,
{
  #[inline]
  fn default() -> Self {
    Self::ok(MD::default())
  }
}

impl<MD> AsMut<ResBuilder<MD>> for ResBuilder<MD> {
  #[inline]
  fn as_mut(&mut self) -> &mut ResBuilder<MD> {
    self
  }
}

impl<MD> AsRef<ResBuilder<MD>> for ResBuilder<MD> {
  #[inline]
  fn as_ref(&self) -> &ResBuilder<MD> {
    self
  }
}

impl<MD> Deref for ResBuilder<MD> {
  type Target = MsgBuilder<MD>;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.msg_builder
  }
}

impl<MD> DerefMut for ResBuilder<MD> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.msg_builder
  }
}
