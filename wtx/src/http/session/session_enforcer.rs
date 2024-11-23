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
pub struct SessionEnforcer<CS, const N: usize> {
  denied: [&'static str; N],
  phantom: PhantomData<CS>,
}

impl<CS, const N: usize> SessionEnforcer<CS, N> {
  /// Creates a new instance with paths that are not taken into consideration.
  #[inline]
  pub fn new(denied: [&'static str; N]) -> Self {
    Self { denied, phantom: PhantomData }
  }
}

impl<CA, CS, E, SA, const N: usize> Middleware<CA, E, SA> for SessionEnforcer<CS, N>
where
  CA: LeaseMut<Option<SessionState<CS>>>,
  E: From<crate::Error>,
{
  type Aux = ();

  #[inline]
  fn aux(&self) -> Self::Aux {
    ()
  }

  #[inline]
  async fn req(
    &self,
    ca: &mut CA,
    _: &mut Self::Aux,
    req: &mut Request<ReqResBuffer>,
    _: &mut SA,
  ) -> Result<ControlFlow<StatusCode, ()>, E> {
    let path = req.rrd.uri.path();
    if self.denied.iter().all(|elem| *elem != path) {
      return Ok(ControlFlow::Continue(()));
    }
    if ca.lease_mut().is_none() {
      return Err(crate::Error::from(SessionError::RequiredSessionInPath).into());
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
