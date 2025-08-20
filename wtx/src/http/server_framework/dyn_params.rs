use crate::http::StatusCode;

/// When returned by an endpoint, perform different types of operations
#[derive(Debug)]
pub enum DynParams {
  /// Clears the body and the headers of the request.
  ClearAll(StatusCode),
  /// Clears the body of the request.
  NoBody(StatusCode),
  /// Clears the headers of the request.
  NoHeaders(StatusCode),
  /// Does not modify the parameters of a request
  Verbatim(StatusCode),
}

#[cfg(feature = "http-server-framework")]
mod http_server_framework {
  use crate::http::{
    ReqResBuffer, Request, StatusCode,
    server_framework::{DynParams, ResFinalizer},
  };

  impl<E> ResFinalizer<E> for DynParams
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn finalize_response(self, req: &mut Request<ReqResBuffer>) -> Result<StatusCode, E> {
      Ok(match self {
        DynParams::ClearAll(elem) => {
          req.rrd.clear();
          elem
        }
        DynParams::NoBody(elem) => {
          req.rrd.body.clear();
          elem
        }
        DynParams::NoHeaders(elem) => {
          req.rrd.headers.clear();
          elem
        }
        DynParams::Verbatim(elem) => elem,
      })
    }
  }
}
