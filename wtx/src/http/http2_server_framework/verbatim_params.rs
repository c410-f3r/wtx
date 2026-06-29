use crate::http::StatusCode;

/// Does not modify the parameters of a request that will be used to form a response. Users
/// should carefully handle incoming and outgoing data.
#[derive(Clone, Copy, Debug)]
pub struct VerbatimParams(
  /// Status code of the response
  pub StatusCode,
);

impl Default for VerbatimParams {
  #[inline]
  fn default() -> Self {
    Self(StatusCode::Ok)
  }
}

#[cfg(feature = "http2-server-framework")]
mod http_server_framework {
  use crate::http::{
    MsgBufferString, Request, StatusCode,
    http2_server_framework::{ResFinalizer, VerbatimParams},
  };

  impl<E> ResFinalizer<E> for VerbatimParams
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn finalize_response(self, _: &mut Request<MsgBufferString>) -> Result<StatusCode, E> {
      Ok(self.0)
    }
  }
}
