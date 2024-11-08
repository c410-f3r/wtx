use crate::{
  http::{
    cookie::{decrypt, CookieBytes},
    server_framework::Middleware,
    KnownHeaderName, ReqResBuffer, Request, Response, Session, SessionError, SessionManagerInner,
    SessionState, SessionStore, StatusCode,
  },
  misc::{GenericTime, LeaseMut, Lock},
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
  session: Session<I, S>,
}

impl<I, S> SessionDecoder<I, S> {
  /// New instance
  #[inline]
  pub fn new(session: Session<I, S>) -> Self {
    Self { session }
  }
}

impl<CA, CS, E, I, RM, SA, S> Middleware<CA, E, SA> for SessionDecoder<I, S>
where
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
    _: &mut CA,
    _: &mut Self::Aux,
    req: &mut Request<ReqResBuffer>,
    _: &mut SA,
  ) -> Result<ControlFlow<StatusCode, ()>, E> {
    let SessionManagerInner { cookie_def, key, state, .. } =
      &mut *self.session.manager.inner.lock().await;
    if let Some(elem) = state {
      if let Some(expire) = &elem.expire {
        let millis = i64::try_from(GenericTime::timestamp()?.as_millis()).unwrap_or_default();
        let date_time = DateTime::from_timestamp_millis(millis).unwrap_or_default();
        if expire >= &date_time {
          let _rslt = self.session.store.get(&(), &()).await?.lease_mut().delete(&elem.id).await;
          return Err(crate::Error::from(SessionError::ExpiredSession).into());
        }
      }
      return Ok(ControlFlow::Continue(()));
    }
    let lease = req.rrd.lease_mut();
    let (vector, headers) = (&mut lease.body, &mut lease.headers);
    for header in headers.iter() {
      if header.name != <&[u8]>::from(KnownHeaderName::Cookie) {
        continue;
      }
      let cookie = CookieBytes::parse(header.value, &mut cookie_def.value)?;
      if cookie.generic.name != cookie_def.name {
        continue;
      }
      let idx = vector.len();
      vector.extend_from_copyable_slice(&cookie_def.value).map_err(Into::into)?;
      cookie_def.value.clear();
      let dec_rslt = decrypt(
        &mut cookie_def.value,
        key,
        (cookie_def.name, vector.get(idx..).unwrap_or_default()),
      );
      vector.truncate(idx);
      dec_rslt?;
      let rslt_des = serde_json::from_slice(&cookie_def.value).map_err(Into::into);
      cookie_def.value.clear();
      let state_des: SessionState<CS> = rslt_des?;
      let state_db_opt =
        self.session.store.get(&(), &()).await?.lease_mut().read(&state_des.id).await?;
      let Some(state_db) = state_db_opt else {
        return Err(crate::Error::from(SessionError::MissingStoredSession).into());
      };
      if state_db != state_des {
        self.session.store.get(&(), &()).await?.lease_mut().delete(&state_des.id).await?;
        return Err(crate::Error::from(SessionError::InvalidStoredSession).into());
      }
      *state = Some(state_des);
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
