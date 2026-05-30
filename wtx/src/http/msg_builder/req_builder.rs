use crate::http::{Method, MsgBuilder, MsgDataMut, Request, WTX_USER_AGENT};
use core::ops::{Deref, DerefMut};

/// Request builder
///
/// Provides shortcuts to manipulate requests through a fluent interface.
#[derive(Debug)]
pub struct ReqBuilder<MD> {
  method: Method,
  msg_builder: MsgBuilder<MD>,
}

impl<MD> ReqBuilder<MD> {
  /// Constructor shortcut that has a default `GET` method
  #[inline]
  pub const fn get(msg_data: MD) -> Self {
    Self::new(Method::Get, msg_data)
  }

  /// Constructor shortcut that has the provided method.
  #[inline]
  pub const fn new(method: Method, msg_data: MD) -> Self {
    Self { method, msg_builder: MsgBuilder::new(msg_data) }
  }

  /// Constructor shortcut that has a default `POST` method
  #[inline]
  pub const fn post(msg_data: MD) -> Self {
    Self::new(Method::Post, msg_data)
  }

  /// Shortcut that converts this instance into a [`Request`].
  #[inline]
  pub fn into_request(self) -> Request<MD> {
    Request::new(self.method, self.msg_builder.msg_data)
  }

  /// Changes the method
  #[inline]
  pub const fn set_method(&mut self, method: Method) -> &mut Self {
    self.method = method;
    self
  }
}

impl<MD> ReqBuilder<MD>
where
  MD: MsgDataMut,
{
  /// Adds the headers that most servers expect from clients: `accept`, `host` and `user-agent`.
  #[inline]
  pub fn basic_req_headers(&mut self) -> crate::Result<&mut Self> {
    let _ = self.accept(&["*/*"])?.host::<()>(None)?.user_agent(&[WTX_USER_AGENT])?;
    Ok(self)
  }
}

impl<'msg_data, MD> ReqBuilder<&'msg_data mut MD> {
  /// Constructor shortcut that has a default `GET` method
  #[inline]
  pub const fn from_req_mut(req: &'msg_data mut Request<MD>) -> Self {
    Self::new(req.method, &mut req.msg_data)
  }
}

impl<MD> AsMut<ReqBuilder<MD>> for ReqBuilder<MD> {
  #[inline]
  fn as_mut(&mut self) -> &mut ReqBuilder<MD> {
    self
  }
}

impl<MD> AsRef<ReqBuilder<MD>> for ReqBuilder<MD> {
  #[inline]
  fn as_ref(&self) -> &ReqBuilder<MD> {
    self
  }
}

impl<MD> Default for ReqBuilder<MD>
where
  MD: Default,
{
  #[inline]
  fn default() -> Self {
    Self::get(MD::default())
  }
}

impl<MD> Deref for ReqBuilder<MD> {
  type Target = MsgBuilder<MD>;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.msg_builder
  }
}

impl<MD> DerefMut for ReqBuilder<MD> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.msg_builder
  }
}
