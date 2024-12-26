use crate::http::{server_framework::StateClean, ReqResBuffer, Request, StatusCode};

/// Modifies responses
pub trait ResFinalizer<E> {
  /// Finalize response
  fn finalize_response(self, req: &mut Request<ReqResBuffer>) -> Result<StatusCode, E>;
}

impl<E> ResFinalizer<E> for ()
where
  E: From<crate::Error>,
{
  #[inline]
  fn finalize_response(self, _: &mut Request<ReqResBuffer>) -> Result<StatusCode, E> {
    Ok(StatusCode::Ok)
  }
}

impl<CA, E, SA> ResFinalizer<E> for (StateClean<'_, CA, SA, ReqResBuffer>, StatusCode)
where
  E: From<crate::Error>,
{
  #[inline]
  fn finalize_response(self, _: &mut Request<ReqResBuffer>) -> Result<StatusCode, E> {
    self.0.req.rrd.clear();
    Ok(self.1)
  }
}

impl<CA, E, SA> ResFinalizer<E> for StateClean<'_, CA, SA, ReqResBuffer>
where
  E: From<crate::Error>,
{
  #[inline]
  fn finalize_response(self, _: &mut Request<ReqResBuffer>) -> Result<StatusCode, E> {
    self.req.rrd.clear();
    Ok(StatusCode::Ok)
  }
}

impl<E> ResFinalizer<E> for StatusCode
where
  E: From<crate::Error>,
{
  #[inline]
  fn finalize_response(self, _: &mut Request<ReqResBuffer>) -> Result<StatusCode, E> {
    Ok(self)
  }
}

impl<E> ResFinalizer<E> for &'static str
where
  E: From<crate::Error>,
{
  #[inline]
  fn finalize_response(self, req: &mut Request<ReqResBuffer>) -> Result<StatusCode, E> {
    req.rrd.body.extend_from_copyable_slice(self.as_bytes())?;
    Ok(StatusCode::Ok)
  }
}

impl<E, T> ResFinalizer<E> for Result<T, E>
where
  E: From<crate::Error>,
  T: ResFinalizer<E>,
{
  #[inline]
  fn finalize_response(self, req: &mut Request<ReqResBuffer>) -> Result<StatusCode, E> {
    self.and_then(|elem| elem.finalize_response(req))
  }
}
