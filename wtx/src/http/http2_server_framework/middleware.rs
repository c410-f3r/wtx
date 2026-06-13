use crate::http::{MsgBufferString, Request, Response, StatusCode};
use core::ops::ControlFlow;

/// Request middleware
pub trait Middleware<D, ER>
where
  ER: From<crate::Error>,
{
  /// Auxiliary structure
  type Aux;

  /// Auxiliary structure
  fn aux(&self) -> Self::Aux;

  /// Modifies or halts requests.
  fn req(
    &self,
    data: &mut D,
    mw_aux: &mut Self::Aux,
    req: &mut Request<MsgBufferString>,
  ) -> impl Future<Output = Result<ControlFlow<StatusCode, ()>, ER>>;

  /// Modifies or halts responses.
  fn res(
    &self,
    data: &mut D,
    mw_aux: &mut Self::Aux,
    res: Response<&mut MsgBufferString>,
  ) -> impl Future<Output = Result<ControlFlow<StatusCode, ()>, ER>>;
}
