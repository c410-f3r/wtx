use crate::{
  http::{server_framework::StateClean, ReqResDataMut, Request, StatusCode},
  misc::Vector,
};

/// Modifies responses
pub trait ResFinalizer<E, RRD> {
  /// Finalize response
  fn finalize_response(self, req: &mut Request<RRD>) -> Result<StatusCode, E>;
}

impl<E, RRD> ResFinalizer<E, RRD> for ()
where
  E: From<crate::Error>,
{
  #[inline]
  fn finalize_response(self, _: &mut Request<RRD>) -> Result<StatusCode, E> {
    Ok(StatusCode::Ok)
  }
}

impl<'any, CA, E, RA, RRD> ResFinalizer<E, RRD> for (StateClean<'any, CA, RA, RRD>, StatusCode)
where
  E: From<crate::Error>,
  RRD: ReqResDataMut,
{
  #[inline]
  fn finalize_response(self, _: &mut Request<RRD>) -> Result<StatusCode, E> {
    Ok(self.1)
  }
}

impl<'any, CA, E, RA, RRD> ResFinalizer<E, RRD> for StateClean<'any, CA, RA, RRD>
where
  E: From<crate::Error>,
  RRD: ReqResDataMut,
{
  #[inline]
  fn finalize_response(self, _: &mut Request<RRD>) -> Result<StatusCode, E> {
    Ok(StatusCode::Ok)
  }
}

impl<E, RRD> ResFinalizer<E, RRD> for StatusCode
where
  E: From<crate::Error>,
  RRD: ReqResDataMut,
{
  #[inline]
  fn finalize_response(self, _: &mut Request<RRD>) -> Result<StatusCode, E> {
    Ok(self)
  }
}

impl<E, RRD> ResFinalizer<E, RRD> for &'static str
where
  E: From<crate::Error>,
  RRD: ReqResDataMut<Body = Vector<u8>>,
{
  #[inline]
  fn finalize_response(self, req: &mut Request<RRD>) -> Result<StatusCode, E> {
    req.rrd.body_mut().extend_from_slice(self.as_bytes()).map_err(From::from)?;
    Ok(StatusCode::Ok)
  }
}

impl<E, RRD, T> ResFinalizer<E, RRD> for Result<T, E>
where
  E: From<crate::Error>,
  T: ResFinalizer<E, RRD>,
{
  #[inline]
  fn finalize_response(self, req: &mut Request<RRD>) -> Result<StatusCode, E> {
    self.and_then(|elem| elem.finalize_response(req))
  }
}
