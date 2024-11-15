use crate::{
  http::{
    server_framework::Middleware, ReqResBuffer, Request, Response, Session, SessionError,
    SessionManagerInner, StatusCode,
  },
  misc::Lock,
};
use core::ops::ControlFlow;

/// Enforces stored session in all requests.
///
///
#[derive(Debug)]
pub struct SessionEnforcer<I, S, const N: usize> {
  denied: [&'static str; N],
  session: Session<I, S>,
}

impl<I, S, const N: usize> SessionEnforcer<I, S, N> {
  /// Creates a new instance with paths that are not taken into consideration.
  #[inline]
  pub fn new(denied: [&'static str; N], session: Session<I, S>) -> Self {
    Self { denied, session }
  }
}

impl<CA, CS, E, I, S, SA, const N: usize> Middleware<CA, E, SA> for SessionEnforcer<I, S, N>
where
  E: From<crate::Error>,
  I: Lock<Resource = SessionManagerInner<CS, E>>,
{
  type Aux = ();

  #[inline]
  fn aux(&self) -> Self::Aux {
    ()
  }

  #[inline]
  async fn req(
    &self,
    _: &mut CA,
    _: &mut Self::Aux,
    req: &mut Request<ReqResBuffer>,
    _: &mut SA,
  ) -> Result<ControlFlow<StatusCode, ()>, E> {
    let path = req.rrd.uri.path();
    if self.denied.iter().all(|elem| *elem != path) {
      return Ok(ControlFlow::Continue(()));
    }
    if self.session.manager.inner.lock().await.state().is_none() {
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
