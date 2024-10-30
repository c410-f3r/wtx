use crate::{
  http::{server_framework::ResFinalizer, ReqResBuffer, Request, StatusCode},
  misc::{FnFut, FnFutWrapper},
};
use core::future::Future;

/// Endpoint that generates a response.
pub trait Endpoint<CA, E, SA> {
  /// Calls endpoint logic
  fn call(
    &self,
    conn_aux: &mut CA,
    path_defs: (u8, &[(&'static str, u8)]),
    req: &mut Request<ReqResBuffer>,
    stream_aux: &mut SA,
  ) -> impl Future<Output = Result<StatusCode, E>>;
}

impl<CA, E, F, SA, RES> Endpoint<CA, E, SA> for FnFutWrapper<(), F>
where
  F: FnFut<(), Result = RES>,
  RES: ResFinalizer<E>,
{
  #[inline]
  async fn call(
    &self,
    _: &mut CA,
    _: (u8, &[(&'static str, u8)]),
    req: &mut Request<ReqResBuffer>,
    _: &mut SA,
  ) -> Result<StatusCode, E> {
    req.rrd.clear();
    self.0.call(()).await.finalize_response(req)
  }
}
