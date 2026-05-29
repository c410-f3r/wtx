use crate::http::{Method, ReqResBuilder, ReqResDataMut, Request, Version, WTX_USER_AGENT};
use core::ops::{Deref, DerefMut};

/// Request builder
///
/// Provides shortcuts to manipulate requests through a fluent interface.
#[derive(Debug)]
pub struct ReqBuilder<RRD> {
  method: Method,
  rrb: ReqResBuilder<RRD>,
  version: Version,
}

impl<RRD> ReqBuilder<RRD> {
  /// Constructor shortcut that has a default `GET` method
  #[inline]
  pub const fn get(rrd: RRD) -> Self {
    Self::new(Method::Get, ReqResBuilder::new(rrd))
  }

  /// Constructor shortcut that has the provided method.
  #[inline]
  pub const fn method(method: Method, rrd: RRD) -> Self {
    Self::new(method, ReqResBuilder::new(rrd))
  }

  /// Constructor shortcut that has a default `POST` method
  #[inline]
  pub const fn post(rrd: RRD) -> Self {
    Self::new(Method::Post, ReqResBuilder::new(rrd))
  }

  /// Shortcut that converts this instance into a [`Request`].
  #[inline]
  pub fn into_request(self) -> Request<RRD> {
    Request::new(self.method, self.rrb.rrd, self.version)
  }

  /// Changes the method
  #[inline]
  pub const fn set_method(&mut self, method: Method) -> &mut Self {
    self.method = method;
    self
  }

  #[inline]
  const fn new(method: Method, rrb: ReqResBuilder<RRD>) -> Self {
    Self { method, rrb, version: Version::Http2 }
  }
}

impl<RRD> ReqBuilder<RRD>
where
  RRD: ReqResDataMut,
{
  /// Adds the headers that most servers expect from clients: `accept`, `host` and `user-agent`.
  #[inline]
  pub fn basic_req_headers(&mut self) -> crate::Result<&mut Self> {
    let _ = self.accept(&["*/*"])?.host::<()>(None)?.user_agent(&[WTX_USER_AGENT])?;
    Ok(self)
  }
}

impl<'rrd, RRD> ReqBuilder<&'rrd mut RRD> {
  /// Constructor shortcut that has a default `GET` method
  #[inline]
  pub const fn from_req_mut(req: &'rrd mut Request<RRD>) -> Self {
    let mut this = Self::new(req.method, ReqResBuilder::new(&mut req.rrd));
    this.version = req.version;
    this
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
