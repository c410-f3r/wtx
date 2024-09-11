use crate::{
  http::{server_framework::ResFinalizer, ReqResDataMut, Request, StatusCode},
  misc::{FnFut, FnFutWrapper},
};
use core::future::Future;

/// Endpoint that generates a response.
pub trait Endpoint<CA, E, RA, RRD> {
  /// Calls endpoint logic
  fn call(
    &self,
    ca: &mut CA,
    path_defs: (u8, &[(&'static str, u8)]),
    ra: &mut RA,
    req: &mut Request<RRD>,
  ) -> impl Future<Output = Result<StatusCode, E>>;
}

impl<CA, E, F, RA, RES, RRD> Endpoint<CA, E, RA, RRD> for FnFutWrapper<(), F>
where
  F: FnFut<(), Result = RES>,
  RES: ResFinalizer<E, RRD>,
  RRD: ReqResDataMut,
{
  #[inline]
  async fn call(
    &self,
    _: &mut CA,
    _: (u8, &[(&'static str, u8)]),
    _: &mut RA,
    req: &mut Request<RRD>,
  ) -> Result<StatusCode, E> {
    req.rrd.clear();
    self.0.call(()).await.finalize_response(req)
  }
}
