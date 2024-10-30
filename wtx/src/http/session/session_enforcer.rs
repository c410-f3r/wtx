use core::marker::PhantomData;

use crate::{
  http::{
    server_framework::ReqMiddleware, ReqResBuffer, ReqResData, Request, Session, SessionError,
    SessionInner,
  },
  misc::{Lease, Lock},
};

/// Enforces stored session in all requests.
///
///
#[derive(Debug)]
pub struct SessionEnforcer<L, SS, const N: usize> {
  denied: [&'static str; N],
  phantom: PhantomData<(L, SS)>,
}

impl<L, SS, const N: usize> SessionEnforcer<L, SS, N> {
  /// Creates a new instance with paths that are not taken into consideration.
  #[inline]
  pub fn new(denied: [&'static str; N]) -> Self {
    Self { denied, phantom: PhantomData }
  }
}

impl<CA, CS, E, L, SA, SS, const N: usize> ReqMiddleware<CA, E, SA> for SessionEnforcer<L, SS, N>
where
  CA: Lease<Session<L, SS>>,
  E: From<crate::Error>,
  L: Lock<Resource = SessionInner<CS, E>>,
{
  #[inline]
  async fn apply_req_middleware(
    &self,
    conn_aux: &mut CA,
    req: &mut Request<ReqResBuffer>,
    _: &mut SA,
  ) -> Result<(), E> {
    let uri = req.rrd.uri();
    let path = uri.path();
    if self.denied.iter().all(|elem| *elem != path) {
      return Ok(());
    }
    if conn_aux.lease().content.lock().await.state().is_none() {
      return Err(crate::Error::from(SessionError::RequiredSessionInPath).into());
    }
    Ok(())
  }
}
