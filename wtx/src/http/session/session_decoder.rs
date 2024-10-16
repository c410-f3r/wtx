use crate::{
  http::{
    cookie::{decrypt, CookieBytes},
    server_framework::ReqMiddleware,
    KnownHeaderName, ReqResBuffer, Request, Session, SessionError, SessionInner, SessionState,
    SessionStore,
  },
  misc::{GenericTime, LeaseMut, Lock},
};
use chrono::DateTime;
use core::marker::PhantomData;
use serde::de::DeserializeOwned;

/// Decodes cookies received from requests and manages them.
///
/// The use of this structure without [`Session`] or used after the applicability of [`Session`]
/// is a NO-OP.
#[derive(Debug)]
pub struct SessionDecoder<L, SS> {
  phantom: PhantomData<(L, SS)>,
}

impl<L, SS> SessionDecoder<L, SS> {
  /// New instance
  #[inline]
  pub fn new() -> Self {
    Self { phantom: PhantomData }
  }
}

impl<CA, CS, E, L, RA, SS> ReqMiddleware<CA, E, RA> for SessionDecoder<L, SS>
where
  CA: LeaseMut<Session<L, SS>>,
  CS: DeserializeOwned + PartialEq,
  E: From<crate::Error>,
  L: Lock<Resource = SessionInner<CS, E>>,
  SS: SessionStore<CS, E>,
{
  #[inline]
  async fn apply_req_middleware(
    &self,
    ca: &mut CA,
    _: &mut RA,
    req: &mut Request<ReqResBuffer>,
  ) -> Result<(), E> {
    let Session { content, store } = ca.lease_mut();
    let SessionInner { cookie_def, phantom: _, key, state } = &mut *content.lock().await;
    if let Some(elem) = state {
      if let Some(expire) = &elem.expire {
        let millis = i64::try_from(GenericTime::timestamp()?.as_millis()).unwrap_or_default();
        let date_time = DateTime::from_timestamp_millis(millis).unwrap_or_default();
        if expire >= &date_time {
          let _rslt = store.delete(&elem.id).await;
          return Err(crate::Error::from(SessionError::ExpiredSession).into());
        }
      }
      return Ok(());
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
      let state_db_opt = store.read(&state_des.id).await?;
      let Some(state_db) = state_db_opt else {
        return Err(crate::Error::from(SessionError::MissingStoredSession).into());
      };
      if state_db != state_des {
        store.delete(&state_des.id).await?;
        return Err(crate::Error::from(SessionError::InvalidStoredSession).into());
      }
      *state = Some(state_des);
      break;
    }
    Ok(())
  }
}

impl<L, SS> Default for SessionDecoder<L, SS> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}
