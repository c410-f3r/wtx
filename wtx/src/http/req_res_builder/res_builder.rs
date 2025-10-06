use crate::http::{ReqResBuilder, Response, StatusCode, Version};
use core::ops::{Deref, DerefMut};

/// Response builder
///
/// Provides shortcuts to manipulate requests through a fluent interface.
///
/// It is also possible to work directly with fields.
#[derive(Debug)]
pub struct ResBuilder<RRD> {
  /// Generic builder
  pub rrb: ReqResBuilder<RRD>,
  /// Status code
  pub status_code: StatusCode,
  /// See [`Version`].
  pub version: Version,
}

impl<RRD> ResBuilder<RRD> {
  /// A new empty response with a 200 OK status code.
  #[inline]
  pub const fn ok(rrd: RRD) -> Self {
    Self { rrb: ReqResBuilder::new(rrd), status_code: StatusCode::Ok, version: Version::Http2 }
  }

  /// Shortcut that converts this instance into a [`Response`].
  #[inline]
  pub fn into_response(self) -> Response<RRD> {
    Response::new(self.rrb.rrd, self.status_code, self.version)
  }

  /// Changes the status code
  #[inline]
  pub const fn status_code(&mut self, status_code: StatusCode) -> &mut Self {
    self.status_code = status_code;
    self
  }
}

impl<RRD> Default for ResBuilder<RRD>
where
  RRD: Default,
{
  #[inline]
  fn default() -> Self {
    Self::ok(RRD::default())
  }
}

impl<RRD> AsMut<ResBuilder<RRD>> for ResBuilder<RRD> {
  #[inline]
  fn as_mut(&mut self) -> &mut ResBuilder<RRD> {
    self
  }
}

impl<RRD> AsRef<ResBuilder<RRD>> for ResBuilder<RRD> {
  #[inline]
  fn as_ref(&self) -> &ResBuilder<RRD> {
    self
  }
}

impl<RRD> Deref for ResBuilder<RRD> {
  type Target = ReqResBuilder<RRD>;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.rrb
  }
}

impl<RRD> DerefMut for ResBuilder<RRD> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.rrb
  }
}
