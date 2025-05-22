use crate::http::{ReqResBuffer, Request, StatusCode};

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
