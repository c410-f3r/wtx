/// Does not modify the parameters of a request that will be used to form a response. Users
/// should carefully handle incoming and outgoing data.
#[derive(Clone, Copy, Debug)]
pub struct VerbatimParams;

#[cfg(feature = "http-server-framework")]
mod http_server_framework {
  use crate::http::{
    ReqResBuffer, Request, StatusCode,
    server_framework::{ResFinalizer, VerbatimParams},
  };

  impl<E> ResFinalizer<E> for VerbatimParams
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn finalize_response(self, _: &mut Request<ReqResBuffer>) -> Result<StatusCode, E> {
      Ok(StatusCode::Ok)
    }
  }
}
