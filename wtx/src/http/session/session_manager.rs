use crate::{
  http::{
    cookie::{cookie_generic::CookieGeneric, encrypt},
    session::SessionKey,
    Header, Headers, KnownHeaderName, ReqResBuffer, ReqResDataMut, SessionManagerBuilder,
    SessionState, SessionStore,
  },
  misc::{GenericTime, Lease, LeaseMut, Lock, Rng, Vector},
};
use chrono::{DateTime, TimeDelta, Utc};
use core::marker::PhantomData;
use serde::Serialize;

/// [`Session`] backed by `tokio`
#[cfg(feature = "tokio")]
pub type SessionManagerTokio<CS, E> =
  SessionManager<crate::misc::Arc<tokio::sync::Mutex<SessionManagerInner<CS, E>>>>;

/// Manages sessions
#[derive(Clone, Debug)]
pub struct SessionManager<SMI> {
  /// Inner content
  pub inner: SMI,
}

impl<CS, E, SMI> SessionManager<SMI>
where
  E: From<crate::Error>,
  SMI: Lock<Resource = SessionManagerInner<CS, E>>,
{
  /// Allows the specification of custom parameters.
  #[inline]
  pub fn builder() -> SessionManagerBuilder {
    SessionManagerBuilder::new()
  }

  /// Removes the session from the store and also modifies headers.
  #[inline]
  pub async fn delete_session_cookie<RRD, S>(
    &mut self,
    rrd: &mut RRD,
    state: &mut Option<SessionState<CS>>,
    store: &mut S,
  ) -> Result<(), E>
  where
    RRD: ReqResDataMut,
    S: SessionStore<CS, E>,
  {
    let SessionManagerInner { cookie_def, phantom: _, key: _ } = &mut *self.inner.lock().await;
    let Some(elem) = state.take() else {
      return Ok(());
    };
    if elem.expires.is_some() {
      store.delete(&elem.id).await?;
    }
    Self::clear_cookie(cookie_def, rrd.headers_mut())?;
    Ok(())
  }

  /// Saves the session in the store and also modifies headers.
  ///
  /// The `rrd` body is used as a temporary buffer but no existing content is erased.
  #[inline]
  pub async fn set_session_cookie<RNG, RRD, S>(
    &mut self,
    custom_state: CS,
    rng: RNG,
    rrd: &mut RRD,
    store: &mut S,
  ) -> Result<(), E>
  where
    CS: Serialize,
    RNG: Rng,
    RRD: LeaseMut<ReqResBuffer>,
    S: SessionStore<CS, E>,
  {
    let SessionManagerInner { cookie_def, phantom: _, key } = &mut *self.inner.lock().await;
    let id = GenericTime::timestamp()?.as_nanos().to_be_bytes();
    let local_state = match (cookie_def.expires, cookie_def.max_age) {
      (None, None) => SessionState { custom_state, expires: None, id },
      (Some(expires), None) => {
        let local_state = SessionState { custom_state, expires: Some(expires), id };
        store.create(&local_state).await?;
        local_state
      }
      (Some(_), Some(max_age)) | (None, Some(max_age)) => {
        let Some(expires_from_max_age) = TimeDelta::from_std(max_age)
          .ok()
          .and_then(|element| Utc::now().checked_add_signed(element))
        else {
          return Err(crate::Error::GenericTimeNeedsBackend.into());
        };
        let local_state = SessionState { custom_state, expires: Some(expires_from_max_age), id };
        store.create(&local_state).await?;
        local_state
      }
    };
    let idx = rrd.lease().body.len();
    serde_json::to_writer(&mut rrd.lease_mut().body, &local_state).map_err(Into::into)?;
    cookie_def.value.clear();
    let rslt = encrypt(
      &mut cookie_def.value,
      key,
      (cookie_def.name, rrd.lease().body.get(idx..).unwrap_or_default()),
      rng,
    );
    rrd.lease_mut().body.truncate(idx);
    rslt?;
    rrd.lease_mut().headers.push_from_fmt(Header::from_name_and_value(
      KnownHeaderName::SetCookie.into(),
      format_args!("{}", &cookie_def),
    ))?;
    Ok(())
  }

  #[inline]
  pub(crate) fn clear_cookie(
    cookie_def: &mut CookieGeneric<&'static [u8], Vector<u8>>,
    headers: &mut Headers,
  ) -> crate::Result<()> {
    let prev_expires = cookie_def.expires;
    let prev_max_age = cookie_def.max_age;
    cookie_def.expires = Some(DateTime::from_timestamp_nanos(0));
    cookie_def.max_age = None;
    cookie_def.value.clear();
    let rslt = headers.push_from_fmt(Header::from_name_and_value(
      KnownHeaderName::SetCookie.into(),
      format_args!("{cookie_def}"),
    ));
    cookie_def.expires = prev_expires;
    cookie_def.max_age = prev_max_age;
    rslt
  }
}

/// Allows the management of state across requests within a connection.
#[derive(Debug)]
pub struct SessionManagerInner<CS, E> {
  pub(crate) cookie_def: CookieGeneric<&'static [u8], Vector<u8>>,
  pub(crate) key: SessionKey,
  pub(crate) phantom: PhantomData<(CS, E)>,
}
