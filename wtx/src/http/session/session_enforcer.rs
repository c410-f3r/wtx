use crate::{
  http::{
    server_framework::Middleware, ReqResBuffer, Request, Response, SessionError, SessionState,
    StatusCode,
  },
  misc::LeaseMut,
};
use core::{marker::PhantomData, ops::ControlFlow};

/// Enforces stored session in all requests.
#[derive(Debug)]
pub struct SessionEnforcer<CS> {
  phantom: PhantomData<CS>,
}

impl<CS> SessionEnforcer<CS> {
  /// Creates a new instance with paths that are not taken into consideration.
  #[inline]
  pub fn new() -> Self {
    Self { phantom: PhantomData }
  }
}

impl<CA, CS, E, SA> Middleware<CA, E, SA> for SessionEnforcer<CS>
where
  CA: LeaseMut<Option<SessionState<CS>>>,
  E: From<crate::Error>,
{
  type Aux = ();

  #[inline]
  fn aux(&self) -> Self::Aux {}

  #[inline]
  async fn req(
    &self,
    ca: &mut CA,
    _: &mut Self::Aux,
    _: &mut Request<ReqResBuffer>,
    _: &mut SA,
  ) -> Result<ControlFlow<StatusCode, ()>, E> {
    if ca.lease_mut().is_none() {
      return Err(crate::Error::from(SessionError::RequiredSession).into());
    }
    Ok(ControlFlow::Continue(()))
  }

  #[inline]
  async fn res(
    &self,
    _: &mut CA,
    _: &mut Self::Aux,
    _: Response<&mut ReqResBuffer>,
    _: &mut SA,
  ) -> Result<ControlFlow<StatusCode, ()>, E> {
    Ok(ControlFlow::Continue(()))
  }
}
