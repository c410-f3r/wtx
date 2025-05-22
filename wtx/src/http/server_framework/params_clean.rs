/// When returned by an endpoint, clears the request body and header contents.
#[derive(Clone, Copy, Debug)]
pub struct ParamsClean;

#[cfg(feature = "http-server-framework")]
mod http_server_framework {
  use crate::http::{
    ReqResBuffer, Request, StatusCode,
    server_framework::{ParamsClean, ResFinalizer},
  };

  impl<E> ResFinalizer<E> for ParamsClean
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn finalize_response(self, req: &mut Request<ReqResBuffer>) -> Result<StatusCode, E> {
      req.rrd.clear();
      Ok(StatusCode::Ok)
    }
  }

  impl<E> ResFinalizer<E> for (ParamsClean, StatusCode)
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
