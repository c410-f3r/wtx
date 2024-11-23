use crate::{
  http::{
    cookie::{decrypt, CookieBytes},
    server_framework::Middleware,
    KnownHeaderName, ReqResBuffer, Request, Response, SessionError, SessionManager,
    SessionManagerInner, SessionState, SessionStore, StatusCode,
  },
  misc::{GenericTime, Lease, LeaseMut, Lock},
  pool::{Pool, ResourceManager},
};
use chrono::DateTime;
use core::ops::ControlFlow;
use serde::de::DeserializeOwned;

/// Decodes cookies received from requests and manages them.
///
/// The use of this structure without [`Session`] or used after the applicability of [`Session`]
/// is a NO-OP.
#[derive(Debug)]
pub struct SessionDecoder<I, S> {
  session_manager: SessionManager<I>,
  session_store: S,
}

impl<I, S> SessionDecoder<I, S> {
  /// New instance
  #[inline]
  pub fn new(session_manager: SessionManager<I>, session_store: S) -> Self {
    Self { session_manager, session_store }
  }
}

impl<CA, CS, E, I, RM, S, SA> Middleware<CA, E, SA> for SessionDecoder<I, S>
where
  CA: LeaseMut<Option<SessionState<CS>>>,
  CS: DeserializeOwned + PartialEq,
  E: From<crate::Error>,
  I: Lock<Resource = SessionManagerInner<CS, E>>,
  S: Pool<ResourceManager = RM>,
  for<'any> S::GetElem<'any>: LeaseMut<RM::Resource>,
  RM: ResourceManager<CreateAux = (), Error = E, RecycleAux = ()>,
  RM::Resource: SessionStore<CS, E>,
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
    let mut session_guard = self.session_manager.inner.lock().await;
    let SessionManagerInner { cookie_def, key, .. } = &mut *session_guard;
    if let Some(elem) = ca.lease() {
      if let Some(expire) = &elem.expire {
        let millis = i64::try_from(GenericTime::timestamp()?.as_millis()).unwrap_or_default();
        let date_time = DateTime::from_timestamp_millis(millis).unwrap_or_default();
        if expire >= &date_time {
          let _rslt = self.session_store.get(&(), &()).await?.lease_mut().delete(&elem.id).await;
          return Err(crate::Error::from(SessionError::ExpiredSession).into());
        }
      }
      return Ok(ControlFlow::Continue(()));
    }
    for header in req.rrd.headers.iter() {
      if header.name != <&str>::from(KnownHeaderName::Cookie) {
        continue;
      }
      let cookie = CookieBytes::parse(header.value, &mut cookie_def.value)?;
      if cookie.generic.name != cookie_def.name {
        continue;
      }
      let idx = req.rrd.body.len();
      req.rrd.body.extend_from_copyable_slice(&cookie_def.value).map_err(Into::into)?;
      cookie_def.value.clear();
      let dec_rslt = decrypt(
        &mut cookie_def.value,
        key,
        (cookie_def.name, req.rrd.body.get(idx..).unwrap_or_default()),
      );
      req.rrd.body.truncate(idx);
      dec_rslt?;
      let rslt_des = serde_json::from_slice(&cookie_def.value).map_err(Into::into);
      cookie_def.value.clear();
      let state_des: SessionState<CS> = rslt_des?;
      let state_db_opt = {
        let mut guard = self.session_store.get(&(), &()).await?;
        guard.lease_mut().read(&state_des.id).await?
      };
      let Some(state_db) = state_db_opt else {
        return Err(crate::Error::from(SessionError::MissingStoredSession).into());
      };
      if state_db != state_des {
        self.session_store.get(&(), &()).await?.lease_mut().delete(&state_des.id).await?;
        return Err(crate::Error::from(SessionError::InvalidStoredSession).into());
      }
      let session_state: &mut Option<_> = ca.lease_mut();
      *session_state = Some(state_des);
      break;
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
