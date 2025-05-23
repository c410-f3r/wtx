/// When returned by an endpoint, clears the headers of the request.
#[derive(Clone, Copy, Debug)]
pub struct HeadersClean;

#[cfg(feature = "http-server-framework")]
mod http_server_framework {
  use crate::http::{
    ReqResBuffer, Request, StatusCode,
    server_framework::{HeadersClean, ResFinalizer},
  };

  impl<E> ResFinalizer<E> for HeadersClean
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn finalize_response(self, req: &mut Request<ReqResBuffer>) -> Result<StatusCode, E> {
      req.rrd.headers.clear();
      Ok(StatusCode::Ok)
    }
  }
}
