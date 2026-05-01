use crate::{
  calendar::Instant,
  collection::{ArrayString, ArrayStringU8, ArrayVectorU8, Vector},
  crypto::{Aead, Aes128GcmGlobal},
  http::{
    Header, KnownHeaderName, ReqResBuffer, ReqResDataMut, SessionManagerBuilder, SessionState,
    SessionStore, cookie::cookie_generic::CookieGeneric,
  },
  misc::{Lease, LeaseMut, Secret},
  rng::CryptoRng,
  sync::{Arc, AsyncMutex},
};
use alloc::string::String;
use core::{
  fmt::{Debug, Formatter},
  marker::PhantomData,
  str,
};
use serde::Serialize;

/// Manages sessions
#[derive(Debug)]
pub struct SessionManager<CS, E> {
  /// Inner content
  pub inner: Arc<(
    // Used to avoid excessive locks
    ArrayStringU8<15>,
    AsyncMutex<SessionManagerInner<CS, E>>,
  )>,
}

impl<CS, E> SessionManager<CS, E>
where
  E: From<crate::Error>,
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
    self.inner.1.lock().await.delete_session_cookie(rrd, state, store).await
  }

  /// Saves the session in the store and also modifies headers.
  ///
  /// The `rrd` body is used as a temporary buffer but no existing content is erased.
  #[inline]
  pub async fn set_session_cookie<RNG, RRD, S>(
    &mut self,
    custom_state: CS,
    rng: &mut RNG,
    rrd: &mut RRD,
    store: &mut S,
  ) -> Result<(), E>
  where
    CS: Serialize,
    RNG: CryptoRng,
    RRD: LeaseMut<ReqResBuffer>,
    S: SessionStore<CS, E>,
  {
    let inner = &mut *self.inner.1.lock().await;
    let SessionManagerInner { cookie_def, session_secret, .. } = inner;
    let session_csrf =
      ArrayString::from_iterator(rng.ascii_graphic_iter().take(32).map(Into::into))?;
    let session_key =
      ArrayString::from_iterator(rng.ascii_graphic_iter().take(32).map(Into::into))?;
    let local_state = match (cookie_def.expires, cookie_def.max_age) {
      (None, None) => SessionState { session_csrf, custom_state, expires_at: None, session_key },
      (Some(expires_at), None) => {
        let elem = SessionState::new(custom_state, Some(expires_at), session_csrf, session_key);
        store.create(&elem).await?;
        elem
      }
      (Some(_), Some(max_age)) | (None, Some(max_age)) => {
        let expires_at = Instant::now_date_time(0)?
          .add(max_age.try_into()?)
          .map_err(crate::Error::from)?
          .trunc_to_us();
        let elem = SessionState::new(custom_state, Some(expires_at), session_csrf, session_key);
        store.create(&elem).await?;
        elem
      }
    };
    let idx = rrd.lease().body.len();
    serde_json::to_writer(&mut rrd.lease_mut().body, &local_state).map_err(Into::into)?;
    cookie_def.value.clear();
    let enc_rslt = session_secret.peek(&mut ArrayVectorU8::<_, { 16 + 28 }>::new(), |el| {
      Aes128GcmGlobal::encrypt_to_buffer_base64(
        cookie_def.name.as_bytes(),
        &mut cookie_def.value,
        rrd.lease().body.get(idx..).unwrap_or_default(),
        rng,
        el.as_ref().try_into()?,
      )
    });
    rrd.lease_mut().body.truncate(idx);
    let _ = enc_rslt??;
    let headers_rslt = rrd.lease_mut().headers.push_from_fmt(Header::from_name_and_value(
      KnownHeaderName::SetCookie.into(),
      format_args!(
        "{}",
        cookie_def.map_mut(
          move |el| el,
          |el| {
            // SAFETY: `encrypt` filled `cookie_def.value` with Base64, which is ASCII.
            unsafe { str::from_utf8_unchecked(el) }
          }
        )
      ),
    ));
    cookie_def.value.clear();
    headers_rslt?;
    rrd.lease_mut().headers.push_from_iter(Header::from_name_and_value(
      KnownHeaderName::XCsrfToken.into(),
      [local_state.session_csrf.as_str()],
    ))?;
    Ok(())
  }
}

impl<CS, E> Clone for SessionManager<CS, E> {
  #[inline]
  fn clone(&self) -> Self {
    Self { inner: self.inner.clone() }
  }
}

/// Allows the management of state across requests within a connection.
pub struct SessionManagerInner<CS, E> {
  pub(crate) cookie_def: CookieGeneric<String, Vector<u8>>,
  pub(crate) phantom: PhantomData<(CS, E)>,
  pub(crate) session_secret: Secret,
}

impl<CS, E> SessionManagerInner<CS, E>
where
  E: From<crate::Error>,
{
  #[inline]
  pub(crate) async fn delete_session_cookie<RRD, S>(
    &mut self,
    rrd: &mut RRD,
    state: &mut Option<SessionState<CS>>,
    store: &mut S,
  ) -> Result<(), E>
  where
    RRD: ReqResDataMut,
    S: SessionStore<CS, E>,
  {
    if let Some(elem) = state.take() {
      store.delete(&elem.session_key).await?;
    };
    self.cookie_def.delete(rrd.headers_mut())?;
    Ok(())
  }
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
