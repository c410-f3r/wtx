use crate::{
  http::{
    KnownHeaderName, ReqResBuffer, Request, Response, SessionError, SessionManager,
    SessionManagerInner, SessionState, SessionStore, StatusCode,
    cookie::{cookie_bytes::CookieBytes, decrypt},
    server_framework::Middleware,
  },
  misc::{GenericTime, Lease, LeaseMut, Lock},
  pool::{Pool, ResourceManager},
};
use chrono::DateTime;
use core::ops::ControlFlow;
use serde::de::DeserializeOwned;

/// Decodes cookies received from requests and manages them.
#[derive(Debug)]
pub struct SessionDecoder<SMI, SS> {
  session_manager: SessionManager<SMI>,
  session_store: SS,
}

impl<SMI, SS> SessionDecoder<SMI, SS> {
  /// New instance
  #[inline]
  pub fn new(session_manager: SessionManager<SMI>, session_store: SS) -> Self {
    Self { session_manager, session_store }
  }
}

impl<CA, CS, E, RM, SMI, SS, SA> Middleware<CA, E, SA> for SessionDecoder<SMI, SS>
where
  CA: LeaseMut<Option<SessionState<CS>>>,
  CS: DeserializeOwned + PartialEq,
  E: From<crate::Error>,
  SMI: Lock<Resource = SessionManagerInner<CS, E>>,
  SS: Pool<ResourceManager = RM>,
  for<'any> SS::GetElem<'any>: LeaseMut<RM::Resource>,
  RM: ResourceManager<CreateAux = (), Error = E, RecycleAux = ()>,
  RM::Resource: SessionStore<CS, E>,
{
  type Aux = ();

  #[inline]
  fn aux(&self) -> Self::Aux {}

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
      if let Some(expires) = &elem.expires {
        let millis = i64::try_from(GenericTime::now_timestamp()?.as_millis()).unwrap_or_default();
        let date_time = DateTime::from_timestamp_millis(millis).unwrap_or_default();
        if expires >= &date_time {
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
      let ss_des: SessionState<CS> = {
        let idx = req.rrd.body.len();
        let cookie_des = CookieBytes::parse(header.value, &mut req.rrd.body)?;
        if cookie_des.generic.name != cookie_def.name {
          continue;
        }
        let (name, value) = (cookie_des.generic.name, cookie_des.generic.value);
        let decrypt_rslt = decrypt(&mut cookie_def.value, key, (name, value));
        req.rrd.body.truncate(idx);
        let value_json = decrypt_rslt?;
        let json_rslt = serde_json::from_slice(value_json);
        cookie_def.value.clear();
        json_rslt.map_err(Into::into)?
      };
      let ss_db_opt = self.session_store.get(&(), &()).await?.lease_mut().read(&ss_des.id).await?;
      let Some(ss_db) = ss_db_opt else {
        return Err(crate::Error::from(SessionError::MissingStoredSession).into());
      };
      if ss_db.custom_state != ss_des.custom_state {
        self.session_store.get(&(), &()).await?.lease_mut().delete(&ss_des.id).await?;
        return Err(crate::Error::from(SessionError::InvalidStoredSession).into());
      }
      let session_state: &mut Option<_> = ca.lease_mut();
      *session_state = Some(ss_des);
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
