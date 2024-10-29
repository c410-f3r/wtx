use crate::http::{ReqResBuffer, Request};

/// Auxiliary structures for streams or requests.
pub trait StreamAux: Sized {
  /// Initialization
  type Init;

  /// Creates a new instance with [`ReqAux::Init`] as well as with a request.
  fn req_aux(init: Self::Init, req: &mut Request<ReqResBuffer>) -> crate::Result<Self>;
}
