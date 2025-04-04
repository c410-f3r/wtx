use crate::{
  http::{
    Header, Headers, KnownHeaderName, ReqResBuffer, ReqResDataMut, SessionManagerBuilder,
    SessionState, SessionStore,
    cookie::{cookie_generic::CookieGeneric, encrypt},
    session::SessionSecret,
  },
  misc::{ArrayString, Lease, LeaseMut, Lock, Rng, Vector},
};
use chrono::{DateTime, TimeDelta, Utc};
use core::{
  fmt::{Debug, Formatter},
  marker::PhantomData,
};
use serde::Serialize;

/// [`SessionManager`] backed by `tokio`
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
    let SessionManagerInner { cookie_def, .. } = &mut *self.inner.lock().await;
    let Some(elem) = state.take() else {
      return Ok(());
    };
    if elem.expires_at.is_some() {
      store.delete(&elem.session_key).await?;
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
    mut rng: RNG,
    rrd: &mut RRD,
    store: &mut S,
  ) -> Result<(), E>
  where
    CS: Serialize,
    RNG: Rng,
    RRD: LeaseMut<ReqResBuffer>,
    S: SessionStore<CS, E>,
  {
    let inner = &mut *self.inner.lock().await;
    let SessionManagerInner { cookie_def, session_secret, .. } = inner;
    let session_csrf = ArrayString::from_iter(rng.ascii_graphic_iter().take(32))?;
    let session_key = ArrayString::from_iter(rng.ascii_graphic_iter().take(32))?;
    let local_state = match (cookie_def.expires, cookie_def.max_age) {
      (None, None) => SessionState { session_csrf, custom_state, expires_at: None, session_key },
      (Some(expires_at), None) => {
        let elem = SessionState::new(custom_state, Some(expires_at), session_csrf, session_key);
        store.create(&elem).await?;
        elem
      }
      (Some(_), Some(max_age)) | (None, Some(max_age)) => {
        let Some(expires_at) = TimeDelta::from_std(max_age)
          .ok()
          .and_then(|element| Utc::now().checked_add_signed(element))
        else {
          return Err(crate::Error::GenericTimeNeedsBackend.into());
        };
        let elem = SessionState::new(custom_state, Some(expires_at), session_csrf, session_key);
        store.create(&elem).await?;
        elem
      }
    };
    let idx = rrd.lease().body.len();
    serde_json::to_writer(&mut rrd.lease_mut().body, &local_state).map_err(Into::into)?;
    cookie_def.value.clear();
    let enc_rslt = encrypt(
      &mut cookie_def.value,
      session_secret.array()?,
      (cookie_def.name.as_bytes(), rrd.lease().body.get(idx..).unwrap_or_default()),
      rng,
    );
    rrd.lease_mut().body.truncate(idx);
    enc_rslt?;
    let headers_rslt = rrd.lease_mut().headers.push_from_fmt(Header::from_name_and_value(
      KnownHeaderName::SetCookie.into(),
      format_args!("{cookie_def}"),
    ));
    cookie_def.value.clear();
    headers_rslt?;
    rrd.lease_mut().headers.push_from_iter(Header::from_name_and_value(
      KnownHeaderName::XCsrfToken.into(),
      [local_state.session_csrf.as_str()],
    ))?;
    Ok(())
  }

  #[inline]
  pub(crate) fn clear_cookie(
    cookie_def: &mut CookieGeneric<&'static str, Vector<u8>>,
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
pub struct SessionManagerInner<CS, E> {
  pub(crate) cookie_def: CookieGeneric<&'static str, Vector<u8>>,
  pub(crate) phantom: PhantomData<(CS, E)>,
  pub(crate) session_secret: SessionSecret,
}

impl<CS, E> Debug for SessionManagerInner<CS, E> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("SessionManagerInner")
      .field("cookie_def", &self.cookie_def)
      .field("phantom", &self.phantom)
      .finish()
  }
}
