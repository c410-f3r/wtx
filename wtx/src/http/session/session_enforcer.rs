use core::marker::PhantomData;

use crate::{
  http::{
    server_framework::ReqMiddleware, ReqResData, Request, Session, SessionError, SessionInner,
  },
  misc::{Lease, Lock},
};

/// Enforces stored session in all requests.
///
///
#[derive(Debug)]
pub struct SessionEnforcer<L, SS, const N: usize> {
  paths: [&'static str; N],
  phantom: PhantomData<(L, SS)>,
}

impl<L, SS, const N: usize> SessionEnforcer<L, SS, N> {
  /// Creates a new instance with paths that are not taken into consideration.
  #[inline]
  pub fn new(paths: [&'static str; N]) -> Self {
    Self { paths, phantom: PhantomData }
  }
}

impl<CA, CS, E, L, RA, RRD, SS, const N: usize> ReqMiddleware<CA, E, RA, RRD>
  for SessionEnforcer<L, SS, N>
where
  CA: Lease<Session<L, SS>>,
  E: From<crate::Error>,
  L: Lock<Resource = SessionInner<CS, E>>,
  RRD: ReqResData,
{
  #[inline]
  async fn apply_req_middleware(
    &self,
    ca: &mut CA,
    _: &mut RA,
    req: &mut Request<RRD>,
  ) -> Result<(), E> {
    let uri = req.rrd.uri();
    let path = uri.path();
    if self.paths.iter().any(|elem| *elem == path) {
      return Ok(());
    }
    if ca.lease().content.lock().await.state().is_none() {
      return Err(crate::Error::from(SessionError::RequiredSessionInPath).into());
    }
    Ok(())
  }
}
