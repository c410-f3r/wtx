#[cfg(feature = "http2-server-framework")]
use crate::http::cookie::CookieNameTy;
use crate::{
  calendar::{Datetime, Utc},
  collections::{ArrayStrU8, FixedStr, ShortBoxStrU8, SuffixGuard, VectorU16},
  crypto::{Aead as _, Aes128GcmGlobal, gen_aead_nonce},
  http::{
    Header, KnownHeaderName, MsgBufferString, RememberMe, SessionCsrf, SessionState, SessionStore,
    cookie::SetCookieGeneric, session::session_manager_builder::SessionManagerBuilder,
  },
  misc::{AsciiAlphaNumeric, LeaseMut, serde_json_deserialize_from_slice},
  rng::{CryptoRng, Rng},
  secret::SecretArray,
  sync::{Arc, AsyncMutex},
};
use core::{
  fmt::{Debug, Formatter},
  marker::PhantomData,
  str,
  time::Duration,
};
use serde::{Serialize, de::DeserializeOwned};

/// Manages sessions.
pub struct SessionManager<CS, E, RNG> {
  /// See [`SessionManagerInner`].
  pub inner: Arc<SessionManagerInner<CS, E, RNG>>,
}

impl<CS, E, RNG> SessionManager<CS, E, RNG>
where
  E: From<crate::Error>,
  RNG: CryptoRng,
{
  /// Allows the specification of custom parameters.
  #[inline]
  pub fn builder() -> SessionManagerBuilder {
    SessionManagerBuilder::new()
  }

  /// Decodes a session that was previously encrypted by this manager.
  #[inline]
  pub async fn decrypt_session(&self, bytes: &[u8]) -> crate::Result<SessionState<CS>>
  where
    CS: DeserializeOwned,
  {
    let inner = &mut *self.inner.sync.lock().await;
    let cookie_name = inner.cookie_def.name;
    let rslt = Aes128GcmGlobal::decrypt_base64_to_buffer(
      cookie_name.as_bytes(),
      &mut inner.cookie_def.value,
      bytes,
      &*inner.session_secret.peek()?,
    );
    let json_rslt = rslt.and_then(|(slice, _)| serde_json_deserialize_from_slice(slice));
    inner.cookie_def.value.clear();
    json_rslt
  }

  /// Removes the session from the store and also modifies headers.
  #[inline]
  pub async fn delete_session_cookie<MD, S>(
    &self,
    msg_data: &mut MD,
    state: &mut Option<SessionState<CS>>,
    store: &mut S,
  ) -> Result<(), E>
  where
    MD: LeaseMut<MsgBufferString>,
    S: SessionStore<CS, E>,
  {
    self.inner.sync.lock().await.delete_session_cookie(msg_data, state, store).await
  }

  /// Indicates whether the current session needs to be rotated.
  #[inline]
  pub fn needs_rotation(&self, state: &SessionState<CS>) -> crate::Result<bool> {
    let now = now_timestamp()?;
    if now.saturating_add(self.inner.grace_period.as_secs()) >= state.expires_at {
      return Ok(false);
    }
    Ok(next_rotation(state.expires_at_ub, state.expires_at, self.inner.lifetime, now).is_some())
  }

  /// Rotates the session cookie, issuing a new session key regardless of how much of the rotation
  /// window has elapsed.
  ///
  /// The previous session key remains valid for the configured grace period. Rotation is skipped
  /// when the lifetime upper bound has already been reached.
  #[inline]
  pub async fn rotate_session_cookie<MD, S>(
    &self,
    state: &SessionState<CS>,
    msg_data: &mut MD,
    store: &mut S,
  ) -> Result<Option<SessionState<CS>>, E>
  where
    CS: Clone + Serialize,
    MD: LeaseMut<MsgBufferString>,
    S: SessionStore<CS, E>,
  {
    let grace_period = self.inner.grace_period;
    let lifetime = self.inner.lifetime;
    self
      .inner
      .sync
      .lock()
      .await
      .rotate_session_cookie(lifetime, grace_period, msg_data, state, store)
      .await
  }

  /// Saves the session in the store and also modifies headers.
  #[inline]
  pub async fn set_session_cookie<MD, S>(
    &self,
    custom_state: &CS,
    msg_data: &mut MD,
    state: &mut Option<SessionState<CS>>,
    store: &mut S,
  ) -> Result<SessionCsrf, E>
  where
    CS: Clone + Serialize,
    MD: LeaseMut<MsgBufferString>,
    S: SessionStore<CS, E>,
  {
    let lifetime = self.inner.lifetime;
    let prev_session_key = state.as_ref().map(|el| el.session_key);
    let new_session_state = self
      .inner
      .sync
      .lock()
      .await
      .create_session_and_push_headers(lifetime, custom_state, msg_data, store)
      .await?;
    if let Some(elem) = prev_session_key {
      let _rslt = store.delete(&elem).await;
    }
    let session_csrf = new_session_state.session_csrf;
    *state = Some(new_session_state);
    Ok(session_csrf)
  }
}

impl<CS, E, RNG> Clone for SessionManager<CS, E, RNG> {
  #[inline]
  fn clone(&self) -> Self {
    Self { inner: self.inner.clone() }
  }
}

impl<CS, E, RNG> Debug for SessionManager<CS, E, RNG> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("SessionManager").finish()
  }
}

/// Inner content
#[derive(Debug)]
pub struct SessionManagerInner<CS, E, RNG> {
  pub(crate) grace_period: Duration,
  pub(crate) lifetime: Duration,
  #[cfg(feature = "http2-server-framework")]
  pub(crate) session_cookie_name: CookieNameTy,
  pub(crate) sync: AsyncMutex<SessionManagerSync<CS, E, RNG>>,
}

/// Allows the management of state across requests within a connection.
pub struct SessionManagerSync<CS, E, RNG> {
  cookie_def: SetCookieGeneric<ShortBoxStrU8, VectorU16<u8>>,
  lifetime_ub: Duration,
  phantom: PhantomData<(CS, E)>,
  remember_me: Option<RememberMe>,
  rng: RNG,
  session_secret: SecretArray<16>,
}

impl<CS, E, RNG> SessionManagerSync<CS, E, RNG>
where
  E: From<crate::Error>,
  RNG: CryptoRng,
{
  pub(crate) fn new(
    cookie_def: SetCookieGeneric<ShortBoxStrU8, VectorU16<u8>>,
    lifetime_ub: Duration,
    remember_me: Option<RememberMe>,
    rng: RNG,
    session_secret: SecretArray<16>,
  ) -> Self {
    Self { cookie_def, lifetime_ub, phantom: PhantomData, remember_me, rng, session_secret }
  }

  #[inline]
  pub(crate) async fn delete_session_cookie<MD, S>(
    &mut self,
    msg_data: &mut MD,
    state: &mut Option<SessionState<CS>>,
    store: &mut S,
  ) -> Result<(), E>
  where
    MD: LeaseMut<MsgBufferString>,
    S: SessionStore<CS, E>,
  {
    if let Some(elem) = state.take() {
      store.delete(&elem.session_key).await?;
    }
    self.cookie_def.delete(&mut msg_data.lease_mut().headers)?;
    Ok(())
  }

  fn build_session_state(
    &mut self,
    custom_state: &CS,
    lifetime: Duration,
  ) -> crate::Result<SessionState<CS>>
  where
    CS: Clone,
  {
    let now = now_timestamp()?;
    let session_csrf = generic_token(&mut self.rng)?;
    let session_key = generic_token(&mut self.rng)?;
    let expires_at_ub = now.saturating_add(self.lifetime_ub.as_secs());
    let expires_at = clamp_expiry(expires_at_ub, lifetime, now);
    Ok(SessionState {
      custom_state: custom_state.clone(),
      expires_at,
      expires_at_ub,
      session_csrf,
      session_key,
    })
  }

  async fn create_session<S>(
    &mut self,
    lifetime: Duration,
    custom_state: &CS,
    store: &mut S,
  ) -> Result<SessionState<CS>, E>
  where
    CS: Clone,
    S: SessionStore<CS, E>,
  {
    let state = self.build_session_state(custom_state, lifetime)?;
    store.create(&state).await?;
    Ok(state)
  }

  async fn create_session_and_push_headers<MD, S>(
    &mut self,
    lifetime: Duration,
    custom_state: &CS,
    msg_data: &mut MD,
    store: &mut S,
  ) -> Result<SessionState<CS>, E>
  where
    CS: Clone + Serialize,
    MD: LeaseMut<MsgBufferString>,
    S: SessionStore<CS, E>,
  {
    let state = self.create_session(lifetime, custom_state, store).await?;
    let headers_len = msg_data.lease_mut().headers.headers_len();
    if let Err(err) = Self::push_csrf_header(&state, msg_data)
      .and_then(|_| self.push_session_cookie_header(&state, msg_data))
    {
      msg_data.lease_mut().headers.truncate(headers_len);
      let _rslt = store.delete(&state.session_key).await;
      return Err(err.into());
    }
    Ok(state)
  }

  fn push_csrf_header<MD>(state: &SessionState<CS>, msg_data: &mut MD) -> crate::Result<()>
  where
    CS: Serialize,
    MD: LeaseMut<MsgBufferString>,
  {
    msg_data.lease_mut().headers.push(Header::from_name_and_value(
      KnownHeaderName::XCsrfToken.into(),
      state.session_csrf.as_str(),
    ))
  }

  fn push_session_cookie_header<MD>(
    &mut self,
    state: &SessionState<CS>,
    msg_data: &mut MD,
  ) -> crate::Result<()>
  where
    CS: Serialize,
    MD: LeaseMut<MsgBufferString>,
  {
    {
      let mut buffer_guard = SuffixGuard::from(&mut msg_data.lease_mut().body);
      let idx = buffer_guard.inner().len();
      serde_json::to_writer(buffer_guard.inner_mut(), state)?;
      self.cookie_def.value.clear();
      let _encrypted = Aes128GcmGlobal::encrypt_to_buffer_base64(
        self.cookie_def.name.as_bytes(),
        &mut self.cookie_def.value,
        gen_aead_nonce(&mut self.rng),
        buffer_guard.inner().get(idx..).unwrap_or_default(),
        &*self.session_secret.peek()?,
      )?;
    }
    let prev_expires = self.cookie_def.expires;
    let prev_max_age = self.cookie_def.max_age;
    self.cookie_def.expires =
      self.remember_me.as_ref().map(|_| datetime_from_timestamp(state.expires_at)).transpose()?;
    self.cookie_def.max_age = None;
    let value = self.cookie_def.map_mut(
      move |el| el,
      |el| {
        // SAFETY: `encrypt_to_buffer_base64` filled `value` with Base64, which is ASCII.
        unsafe { str::from_utf8_unchecked(el) }
      },
    );
    let cookie_rslt = msg_data.lease_mut().headers.push_from_fmt(Header::from_name_and_value(
      KnownHeaderName::SetCookie.into(),
      format_args!("{value}"),
    ));
    self.cookie_def.expires = prev_expires;
    self.cookie_def.max_age = prev_max_age;
    self.cookie_def.value.clear();
    cookie_rslt
  }

  #[inline]
  async fn rotate_session_cookie<MD, S>(
    &mut self,
    lifetime: Duration,
    grace_period: Duration,
    msg_data: &mut MD,
    state: &SessionState<CS>,
    store: &mut S,
  ) -> Result<Option<SessionState<CS>>, E>
  where
    CS: Clone + Serialize,
    MD: LeaseMut<MsgBufferString>,
    S: SessionStore<CS, E>,
  {
    let now = now_timestamp()?;
    if now >= state.expires_at_ub {
      return Ok(None);
    }
    let new_expires_at = clamp_expiry(state.expires_at_ub, lifetime, now);
    let new_state = SessionState {
      custom_state: state.custom_state.clone(),
      expires_at: new_expires_at,
      expires_at_ub: state.expires_at_ub,
      session_csrf: state.session_csrf,
      session_key: generic_token(&mut self.rng)?,
    };
    let grace_expires_at =
      clamp_expiry(state.expires_at_ub, grace_period, now).min(state.expires_at);
    let mut old_state = state.clone();
    old_state.expires_at = grace_expires_at;
    store.create(&new_state).await?;
    if let Err(err) = store.update(&state.session_key, &old_state).await {
      let _rslt = store.delete(&new_state.session_key).await;
      return Err(err);
    }
    let headers_len = msg_data.lease_mut().headers.headers_len();
    if let Err(err) = self.push_session_cookie_header(&new_state, msg_data) {
      msg_data.lease_mut().headers.truncate(headers_len);
      let _rslt0 = store.delete(&new_state.session_key).await;
      let _rslt1 = store.update(&state.session_key, state).await;
      return Err(err.into());
    }
    Ok(Some(new_state))
  }
}

impl<CS, E, RNG> Debug for SessionManagerSync<CS, E, RNG> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("SessionManagerSync").finish()
  }
}

#[inline]
fn clamp_expiry(expires_at_ub: u64, lifetime: Duration, now: u64) -> u64 {
  let expires_at = now.saturating_add(lifetime.as_secs());
  expires_at.min(expires_at_ub)
}

#[inline]
fn datetime_from_timestamp(timestamp: u64) -> crate::Result<Datetime<Utc>> {
  Ok(Datetime::from_timestamp_secs(i64::try_from(timestamp)?)?)
}

#[inline]
fn generic_token<RNG>(rng: &mut RNG) -> crate::Result<FixedStr<32>>
where
  RNG: Rng,
{
  let iter = rng.ascii_iter(AsciiAlphaNumeric::default()).take(32).map(Into::into);
  ArrayStrU8::from_iterator(iter)?.into_fixed_str()
}

#[inline]
fn next_rotation(expires_at_ub: u64, expires_at: u64, lifetime: Duration, now: u64) -> Option<u64> {
  if now >= expires_at_ub {
    return None;
  }
  if !rotation_is_due(expires_at, lifetime, now) {
    return None;
  }
  let new_expires_at = clamp_expiry(expires_at_ub, lifetime, now);
  (new_expires_at > expires_at).then_some(new_expires_at)
}

#[inline]
fn now_timestamp() -> crate::Result<u64> {
  Ok(u64::try_from(Datetime::now()?.timestamp_secs_and_ns().0)?)
}

#[inline]
fn rotation_is_due(expires_at: u64, lifetime: Duration, now: u64) -> bool {
  let half_life = lifetime.as_secs().div_ceil(2);
  now.checked_add(half_life).is_none_or(|elem| elem >= expires_at)
}
