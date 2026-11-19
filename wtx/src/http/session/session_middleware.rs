use crate::{
  calendar::Datetime,
  collections::{ShortBoxStrU8, SuffixGuard, VectorU16, VectorUsize},
  http::{
    KnownHeaderName, MsgBufferString, Request, Response, SessionCsrf, SessionManager, SessionState,
    SessionStore, StatusCode, cookie::CookieStr, http2_server_framework::Middleware,
  },
  misc::{Lease as _, LeaseMut},
  pool::{ResourceManager, SimplePool},
  rng::CryptoRng,
};
use core::{
  fmt::{Debug, Formatter},
  ops::ControlFlow,
};
use serde::{Serialize, de::DeserializeOwned};

/// Decodes cookies received from requests and manages them.
pub struct SessionMiddleware<CS, E, RM, RNG>
where
  RM: ResourceManager,
{
  allowed_paths: VectorU16<ShortBoxStrU8>,
  session_manager: SessionManager<CS, E, RNG>,
  session_store: SimplePool<RM>,
}

impl<CS, E, RM, RNG> SessionMiddleware<CS, E, RM, RNG>
where
  RM: ResourceManager,
{
  /// New instance
  #[inline]
  pub const fn new(
    allowed_paths: VectorU16<ShortBoxStrU8>,
    session_manager: SessionManager<CS, E, RNG>,
    session_store: SimplePool<RM>,
  ) -> Self {
    Self { allowed_paths, session_manager, session_store }
  }
}

impl<CS, E, RM, RNG> SessionMiddleware<CS, E, RM, RNG>
where
  CS: DeserializeOwned + PartialEq,
  E: From<crate::Error>,
  RM: ResourceManager<CreateAux = (), Error = E, RecycleAux = ()>,
  RM::Resource: SessionStore<CS, E>,
  RNG: CryptoRng,
{
  async fn parse_headers(
    &self,
    req: &mut Request<MsgBufferString>,
    session_name: &str,
  ) -> Result<ParsedHeaders<CS>, E> {
    let mut ph = ParsedHeaders::default();
    for header in &req.msg_data.headers {
      if ph.psc.is_valid() && ph.csrf.is_some() {
        break;
      }
      match header.name {
        elem if elem == <&str>::from(KnownHeaderName::Cookie) => {
          ph.psc = ph.psc.merge(
            self.parse_session_cookie(&mut req.msg_data.body, header.value, session_name).await,
          );
        }
        elem if elem == <&str>::from(KnownHeaderName::XCsrfToken) => {
          ph.csrf = header.value.try_into().ok();
        }
        _ => {}
      }
    }
    Ok(ph)
  }

  async fn parse_session_cookie(
    &self,
    buffer: &mut VectorUsize<u8>,
    header_value: &str,
    session_name: &str,
  ) -> ParsedSessionCookie<CS> {
    let mut guarded = SuffixGuard::from(buffer);
    let Ok(parsed) = CookieStr::parse(guarded.inner_mut(), header_value) else {
      return ParsedSessionCookie::Absent;
    };
    let Some((_, value)) = parsed.0.values.iter().find(|(name, _)| name.as_str() == session_name)
    else {
      return ParsedSessionCookie::Absent;
    };
    match self.validate_session(value.as_bytes()).await {
      Ok(Some(elem)) => ParsedSessionCookie::Valid(elem),
      _ => ParsedSessionCookie::Invalid,
    }
  }

  async fn validate_session(&self, bytes: &[u8]) -> Result<Option<SessionState<CS>>, E> {
    let Ok(des) = self.session_manager.decrypt_session(bytes).await else {
      _trace!("Session cookie could not be decrypted");
      return Ok(None);
    };
    let mut pool_guard = self.session_store.get_with_unit().await?;
    let Some(db) = pool_guard.read(des.session_key).await? else {
      _trace!("Session found in headers does not exist in database");
      return Ok(None);
    };
    if db.custom_state != des.custom_state || db.session_csrf != des.session_csrf {
      _trace!("Session found in headers does not match database session");
      return Ok(None);
    }
    let now =
      u64::try_from(Datetime::now()?.timestamp_secs_and_ns().0).map_err(crate::Error::from)?;
    if should_expire(&db, now) {
      _trace!("Session found in database is expired and will be removed");
      pool_guard.delete(&db.session_key).await?;
      return Ok(None);
    }
    Ok(Some(db))
  }
}

impl<D, CS, E, RM, RNG> Middleware<D, E> for SessionMiddleware<CS, E, RM, RNG>
where
  D: LeaseMut<Option<SessionState<CS>>>,
  CS: Clone + DeserializeOwned + PartialEq + Serialize,
  E: From<crate::Error>,
  RM: ResourceManager<CreateAux = (), Error = E, RecycleAux = ()>,
  RM::Resource: SessionStore<CS, E>,
  RNG: CryptoRng,
{
  type Aux = ();

  #[inline]
  fn aux(&self) -> Self::Aux {}

  /// Iterates over all headers.
  ///
  /// 1. A request can contain several cookies with different names.
  /// 2. `XCsrfToken` might be located after the session cookie.
  #[inline]
  async fn req(
    &self,
    data: &mut D,
    _: &mut Self::Aux,
    req: &mut Request<MsgBufferString>,
  ) -> Result<ControlFlow<StatusCode, ()>, E> {
    let session_name = self.session_manager.inner.session_cookie_name.as_str();
    let ParsedHeaders { csrf, psc } = self.parse_headers(req, session_name).await?;
    // `data` must only reflect the current request, otherwise a request without a session cookie
    // would inherit the state established by a previous request on the same connection.
    *data.lease_mut() = None;
    match psc {
      ParsedSessionCookie::Absent => {
        let path = req.msg_data.uri.path();
        if self.allowed_paths.iter().all(|el| el.as_str() != path) {
          _trace!("Session was not found in headers and path is forbidden");
          req.clear();
          return Ok(ControlFlow::Break(StatusCode::Forbidden));
        }
        _trace!("Session was not found in headers but an allowed path succeeded");
        Ok(ControlFlow::Continue(()))
      }
      ParsedSessionCookie::Invalid => {
        _trace!("Invalid session cookie found, removing it");
        delete_session_cookie(data, req, &self.session_manager, &self.session_store).await?;
        Ok(ControlFlow::Break(StatusCode::Forbidden))
      }
      ParsedSessionCookie::Valid(session_state) => {
        if req.method.is_mutable() && Some(session_state.session_csrf) != csrf {
          _trace!("Session does not contain a valid CSRF");
          *data.lease_mut() = Some(session_state);
          delete_session_cookie(data, req, &self.session_manager, &self.session_store).await?;
          return Ok(ControlFlow::Break(StatusCode::Forbidden));
        }
        _trace!("Session has been successfully validated");
        *data.lease_mut() = Some(session_state);
        Ok(ControlFlow::Continue(()))
      }
    }
  }

  #[inline]
  async fn res(
    &self,
    data: &mut D,
    _: &mut Self::Aux,
    res: Response<&mut MsgBufferString>,
  ) -> Result<ControlFlow<StatusCode, ()>, E> {
    if let Some(elem) = data.lease()
      && self.session_manager.needs_rotation(elem)?
      && let mut pool_guard = self.session_store.get_with_unit().await?
      && let Some(rotated) =
        self.session_manager.rotate_session_cookie(elem, res.msg_data, &mut ***pool_guard).await?
    {
      _trace!("Session cookie has been rotated");
      *data.lease_mut() = Some(rotated);
    }
    Ok(ControlFlow::Continue(()))
  }
}

#[expect(clippy::missing_fields_in_debug, reason = "not sure if the other fields are necessary")]
impl<CS, E, RM, RNG> Debug for SessionMiddleware<CS, E, RM, RNG>
where
  RM: ResourceManager,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("SessionMiddleware").field("allowed_paths", &self.allowed_paths).finish()
  }
}

struct ParsedHeaders<CS> {
  csrf: Option<SessionCsrf>,
  psc: ParsedSessionCookie<CS>,
}

impl<CS> Default for ParsedHeaders<CS> {
  #[inline]
  fn default() -> Self {
    Self { csrf: None, psc: ParsedSessionCookie::Absent }
  }
}

enum ParsedSessionCookie<CS> {
  Absent,
  Invalid,
  Valid(SessionState<CS>),
}

impl<CS> ParsedSessionCookie<CS> {
  #[inline]
  const fn is_valid(&self) -> bool {
    matches!(self, Self::Valid(..))
  }

  #[inline]
  fn merge(self, other: Self) -> Self {
    match (self, other) {
      (Self::Valid(state), _) | (_, Self::Valid(state)) => Self::Valid(state),
      (Self::Invalid, _) | (_, Self::Invalid) => Self::Invalid,
      (Self::Absent, Self::Absent) => Self::Absent,
    }
  }
}

#[inline]
async fn delete_session_cookie<CS, D, E, RM, RNG>(
  data: &mut D,
  req: &mut Request<MsgBufferString>,
  session_manager: &SessionManager<CS, E, RNG>,
  session_store: &SimplePool<RM>,
) -> Result<(), E>
where
  D: LeaseMut<Option<SessionState<CS>>>,
  E: From<crate::Error>,
  RM: ResourceManager<CreateAux = (), Error = E, RecycleAux = ()>,
  RM::Resource: SessionStore<CS, E>,
  RNG: CryptoRng,
{
  req.clear();
  let mut pool_guard = session_store.get_with_unit().await?;
  session_manager
    .delete_session_cookie(&mut req.msg_data, data.lease_mut(), &mut ***pool_guard)
    .await
}

#[inline]
fn should_expire<CS>(state: &SessionState<CS>, now: u64) -> bool {
  state.expires_at <= now || state.expires_at_ub <= now
}
