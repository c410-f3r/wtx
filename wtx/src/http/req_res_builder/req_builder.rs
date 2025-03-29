use crate::http::{Method, ReqResBuilder, Request, Version};
use core::ops::{Deref, DerefMut};

/// Request builder
///
/// Provides shortcuts to manipulate requests through a fluent interface.
///
/// It is also possible to work directly with fields.
#[derive(Debug)]
pub struct ReqBuilder<RRD> {
  /// Method
  pub method: Method,
  /// Generic builder
  pub rrb: ReqResBuilder<RRD>,
  /// Version
  pub version: Version,
}

impl<RRD> ReqBuilder<RRD> {
  /// Constructor shortcut that has a default `GET` method
  #[inline]
  pub const fn get(rrd: RRD) -> Self {
    Self { method: Method::Get, rrb: ReqResBuilder::new(rrd), version: Version::Http2 }
  }

  /// Constructor shortcut that has a default `POST` method
  #[inline]
  pub const fn post(rrd: RRD) -> Self {
    Self { method: Method::Get, rrb: ReqResBuilder::new(rrd), version: Version::Http2 }
  }

  /// Shortcut that converts this instance into a [`Request`].
  #[inline]
  pub fn into_request(self) -> Request<RRD> {
    Request::new(self.method, self.rrb.rrd, self.version)
  }

  /// Changes the method
  #[inline]
  pub fn method(&mut self, method: Method) -> &mut Self {
    self.method = method;
    self
  }
}

impl<RRD> AsMut<ReqBuilder<RRD>> for ReqBuilder<RRD> {
  #[inline]
  fn as_mut(&mut self) -> &mut ReqBuilder<RRD> {
    self
  }
}

impl<RRD> AsRef<ReqBuilder<RRD>> for ReqBuilder<RRD> {
  #[inline]
  fn as_ref(&self) -> &ReqBuilder<RRD> {
    self
  }
}

impl<RRD> Default for ReqBuilder<RRD>
where
  RRD: Default,
{
  #[inline]
  fn default() -> Self {
    Self::get(RRD::default())
  }
}

impl<RRD> Deref for ReqBuilder<RRD> {
  type Target = ReqResBuilder<RRD>;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.rrb
  }
}

impl<RRD> DerefMut for ReqBuilder<RRD> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.rrb
  }
}
