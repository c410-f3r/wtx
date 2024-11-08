use crate::http::{ReqResBuffer, Request, Response, StatusCode};
use core::{future::Future, ops::ControlFlow};

/// Request middleware
pub trait Middleware<CA, E, SA>
where
  E: From<crate::Error>,
{
  /// Auxiliary structure
  type Aux;

  /// Auxiliary structure
  fn aux(&self) -> Self::Aux;

  /// Modifies or halts requests.
  fn req(
    &self,
    conn_aux: &mut CA,
    mw_aux: &mut Self::Aux,
    req: &mut Request<ReqResBuffer>,
    stream_aux: &mut SA,
  ) -> impl Future<Output = Result<ControlFlow<StatusCode, ()>, E>>;

  /// Modifies or halts responses.
  fn res(
    &self,
    conn_aux: &mut CA,
    mw_aux: &mut Self::Aux,
    res: Response<&mut ReqResBuffer>,
    stream_aux: &mut SA,
  ) -> impl Future<Output = Result<ControlFlow<StatusCode, ()>, E>>;
}
