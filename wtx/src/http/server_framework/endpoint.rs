use crate::{
  http::{server_framework::ResFinalizer, ReqResBuffer, Request, StatusCode},
  misc::{FnFut, FnFutWrapper},
};
use core::future::Future;

/// Endpoint that generates a response.
pub trait Endpoint<CA, E, RA> {
  /// Calls endpoint logic
  fn call(
    &self,
    ca: &mut CA,
    path_defs: (u8, &[(&'static str, u8)]),
    ra: &mut RA,
    req: &mut Request<ReqResBuffer>,
  ) -> impl Future<Output = Result<StatusCode, E>>;
}

impl<CA, E, F, RA, RES> Endpoint<CA, E, RA> for FnFutWrapper<(), F>
where
  F: FnFut<(), Result = RES>,
  RES: ResFinalizer<E>,
{
  #[inline]
  async fn call(
    &self,
    _: &mut CA,
    _: (u8, &[(&'static str, u8)]),
    _: &mut RA,
    req: &mut Request<ReqResBuffer>,
  ) -> Result<StatusCode, E> {
    req.rrd.clear();
    self.0.call(()).await.finalize_response(req)
  }
}
