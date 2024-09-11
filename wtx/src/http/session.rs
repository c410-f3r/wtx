mod session_builder;
mod session_decoder;
mod session_enforcer;
mod session_error;
mod session_state;
mod session_store;

use crate::{
  http::{
    cookie::{encrypt, CookieGeneric},
    server_framework::ConnAux,
    Header, KnownHeaderName, ReqResDataMut,
  },
  misc::{GenericTime, Lease, LeaseMut, Lock, Rng, Vector},
};
use chrono::DateTime;
use core::marker::PhantomData;
use serde::Serialize;
pub use session_builder::SessionBuilder;
pub use session_decoder::SessionDecoder;
pub use session_enforcer::SessionEnforcer;
pub use session_error::SessionError;
pub use session_state::SessionState;
pub use session_store::SessionStore;

type SessionId = [u8; 16];
type SessionKey = [u8; 16];
/// [`Session`] backed by `tokio`
#[cfg(feature = "tokio")]
pub type SessionTokio<CS, E, SS> =
  Session<alloc::sync::Arc<tokio::sync::Mutex<SessionInner<CS, E>>>, SS>;

/// Allows the management of state across requests within a connection.
#[derive(Clone, Debug)]
pub struct Session<L, SS> {
  /// Content
  pub content: L,
  /// Store
  pub store: SS,
}

impl<CS, E, L, SS> Session<L, SS>
where
  E: From<crate::Error>,
  L: Lock<Resource = SessionInner<CS, E>>,
  SS: SessionStore<CS, E>,
{
  /// Allows the specification of custom parameters.
  #[inline]
  pub fn builder(key: SessionKey, store: SS) -> SessionBuilder<SS> {
    SessionBuilder::new(key, store)
  }

  /// Removes the session from the store and also modifies headers.
  #[inline]
  pub async fn delete_session_cookie<RRD>(&mut self, rrd: &mut RRD) -> Result<(), E>
  where
    RRD: ReqResDataMut,
  {
    let SessionInner { cookie_def, phantom: _, key: _, state } = &mut *self.content.lock().await;
    if let Some(elem) = state.take() {
      self.store.delete(&elem.id).await?;
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
  #[inline]
  pub async fn set_session_cookie<RNG, RRD>(
    &mut self,
    custom_state: CS,
    rng: RNG,
    rrd: &mut RRD,
  ) -> Result<(), E>
  where
    CS: Serialize,
    RNG: Rng,
    RRD: ReqResDataMut<Body = Vector<u8>>,
  {
    let SessionInner { cookie_def, phantom: _, key, state } = &mut *self.content.lock().await;
    cookie_def.value.clear();
    let id = GenericTime::timestamp().map_err(Into::into)?.as_nanos().to_be_bytes();
    let local_state = if let Some(elem) = cookie_def.expire {
      let local_state = SessionState { custom_state, expire: Some(elem), id };
      self.store.create(&local_state).await?;
      local_state
    } else {
      SessionState { custom_state, expire: None, id }
    };
    let idx = rrd.body().len();
    serde_json::to_writer(rrd.body_mut(), &local_state).map_err(Into::into)?;
    *state = Some(local_state);
    let rslt = encrypt(
      &mut cookie_def.value,
      key,
      (cookie_def.name, rrd.body().get(idx..).unwrap_or_default()),
      rng,
    );
    rrd.body_mut().truncate(idx);
    rslt?;
    rrd.headers_mut().push_from_fmt(Header::from_name_and_value(
      KnownHeaderName::SetCookie.into(),
      format_args!("{}", &cookie_def),
    ))?;
    Ok(())
  }
}

impl<CS, E, L, SS> ConnAux for Session<L, SS>
where
  L: Lock<Resource = SessionInner<CS, E>>,
{
  type Init = Session<L, SS>;

  #[inline]
  fn conn_aux(init: Self::Init) -> crate::Result<Self> {
    Ok(init)
  }
}

impl<L, SS> Lease<Self> for Session<L, SS> {
  #[inline]
  fn lease(&self) -> &Self {
    self
  }
}

impl<L, SS> LeaseMut<Self> for Session<L, SS> {
  #[inline]
  fn lease_mut(&mut self) -> &mut Self {
    self
  }
}

impl<A, L, SS> Lease<Session<L, SS>> for (Session<L, SS>, A) {
  #[inline]
  fn lease(&self) -> &Session<L, SS> {
    &self.0
  }
}

impl<A, L, SS> LeaseMut<Session<L, SS>> for (Session<L, SS>, A) {
  #[inline]
  fn lease_mut(&mut self) -> &mut Session<L, SS> {
    &mut self.0
  }
}

/// Allows the management of state across requests within a connection.
#[derive(Debug)]
pub struct SessionInner<CS, E> {
  cookie_def: CookieGeneric<&'static [u8], Vector<u8>>,
  key: SessionKey,
  phantom: PhantomData<E>,
  state: Option<SessionState<CS>>,
}

impl<CS, E> SessionInner<CS, E> {
  /// State saved in the store or in the current session.
  #[inline]
  pub fn state(&self) -> &Option<SessionState<CS>> {
    &self.state
  }
}
