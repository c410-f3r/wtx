use crate::{
  http::{
    cookie::{encrypt, CookieGeneric},
    session::SessionKey,
    Header, KnownHeaderName, ReqResBuffer, ReqResDataMut, SessionState, SessionStore,
  },
  misc::{GenericTime, Lease, LeaseMut, Lock, Rng, Vector},
};
use chrono::DateTime;
use core::marker::PhantomData;
use serde::Serialize;

/// Manages sessions
#[derive(Clone, Debug)]
pub struct SessionManager<I> {
  /// Inner content
  pub inner: I,
}

impl<CS, E, I> SessionManager<I>
where
  E: From<crate::Error>,
  I: Lock<Resource = SessionManagerInner<CS, E>>,
{
  /// Removes the session from the store and also modifies headers.
  #[inline]
  pub async fn delete_session_cookie<RRD, S>(
    &mut self,
    rrd: &mut RRD,
    store: &mut S,
  ) -> Result<(), E>
  where
    RRD: ReqResDataMut,
    S: SessionStore<CS, E>,
  {
    let SessionManagerInner { cookie_def, phantom: _, key: _, state } =
      &mut *self.inner.lock().await;
    if let Some(elem) = state.take() {
      store.delete(&elem.id).await?;
    }
    cookie_def.expire = Some(DateTime::from_timestamp_nanos(0));
    cookie_def.value.clear();
    rrd.headers_mut().push_from_fmt(Header::from_name_and_value(
      KnownHeaderName::SetCookie.into(),
      format_args!("{cookie_def}"),
    ))?;
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
    let SessionManagerInner { cookie_def, phantom: _, key, state } = &mut *self.inner.lock().await;
    cookie_def.value.clear();
    let id = GenericTime::timestamp().map_err(Into::into)?.as_nanos().to_be_bytes();
    let local_state = if let Some(elem) = cookie_def.expire {
      let local_state = SessionState { custom_state, expire: Some(elem), id };
      store.create(&local_state).await?;
      local_state
    } else {
      SessionState { custom_state, expire: None, id }
    };
    let idx = rrd.lease().body.len();
    serde_json::to_writer(&mut rrd.lease_mut().body, &local_state).map_err(Into::into)?;
    *state = Some(local_state);
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
}

/// Allows the management of state across requests within a connection.
#[derive(Debug)]
pub struct SessionManagerInner<CS, E> {
  pub(crate) cookie_def: CookieGeneric<&'static [u8], Vector<u8>>,
  pub(crate) key: SessionKey,
  pub(crate) phantom: PhantomData<E>,
  pub(crate) state: Option<SessionState<CS>>,
}

impl<CS, E> SessionManagerInner<CS, E> {
  /// State saved in the store or in the current session.
  #[inline]
  pub fn state(&self) -> &Option<SessionState<CS>> {
    &self.state
  }
}
