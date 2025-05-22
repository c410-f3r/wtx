/// When returned by an endpoint, clears the body of the request.
#[derive(Clone, Copy, Debug)]
pub struct BodyClean;

#[cfg(feature = "http-server-framework")]
mod http_server_framework {
  use crate::http::{
    ReqResBuffer, Request, StatusCode,
    server_framework::{BodyClean, ResFinalizer},
  };

  impl<E> ResFinalizer<E> for BodyClean
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn finalize_response(self, req: &mut Request<ReqResBuffer>) -> Result<StatusCode, E> {
      req.rrd.body.clear();
      Ok(StatusCode::Ok)
    }
  }

  impl<E> ResFinalizer<E> for (BodyClean, StatusCode)
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn finalize_response(self, req: &mut Request<ReqResBuffer>) -> Result<StatusCode, E> {
      let _ = self.0.finalize_response(req)?;
      Ok(self.1)
    }
  }
}
