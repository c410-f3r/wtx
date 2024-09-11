use crate::http::{ReqResDataMut, Request};

/// Auxiliary structure for requests
pub trait ReqAux: Sized {
  /// Initialization
  type Init;

  /// Creates a new instance with [`ReqAux::Init`] as well as with a request.
  fn req_aux<RRD>(init: Self::Init, req: &mut Request<RRD>) -> crate::Result<Self>
  where
    RRD: ReqResDataMut;
}
