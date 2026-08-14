use crate::{
  calendar::{DateTime, Instant, Utc},
  collections::{ArrayVectorU8, Vector},
  crypto::{Aead as _, Aes128GcmGlobal},
  http::{
    KnownHeaderName, MsgBufferString, Request, Response, SessionManager, SessionManagerInner,
    SessionState, SessionStore, StatusCode, cookie::cookie_str::CookieStr,
    http2_server_framework::Middleware,
  },
  misc::{Lease as _, LeaseMut, serde_json_deserialize_from_slice},
  pool::{ResourceManager, SimplePool},
};
use alloc::string::String;
use core::ops::ControlFlow;
use serde::de::DeserializeOwned;

/// Decodes cookies received from requests and manages them.
#[derive(Debug)]
pub struct SessionMiddleware<CS, E, RM>
where
  RM: ResourceManager,
{
  allowed_paths: Vector<String>,
  session_manager: SessionManager<CS, E>,
  session_store: SimplePool<RM>,
}

impl<CS, E, RM> SessionMiddleware<CS, E, RM>
where
  RM: ResourceManager,
{
  /// New instance
  #[inline]
  pub const fn new(
    allowed_paths: Vector<String>,
    session_manager: SessionManager<CS, E>,
    session_store: SimplePool<RM>,
  ) -> Self {
    Self { allowed_paths, session_manager, session_store }
  }
}

impl<D, CS, E, RM> Middleware<D, E> for SessionMiddleware<CS, E, RM>
where
  D: LeaseMut<Option<SessionState<CS>>>,
  CS: DeserializeOwned + PartialEq,
  E: From<crate::Error>,
  RM: ResourceManager<CreateAux = (), Error = E, RecycleAux = ()>,
  RM::Resource: SessionStore<CS, E>,
{
  type Aux = ();

  #[inline]
  fn aux(&self) -> Self::Aux {}

  /// Iterates over all headers.
  ///
  /// 1. A request can contain several cookies with different names.
  /// 2. `XCsrfToken` might be located after the desired cookie.
  #[inline]
  async fn req(
    &self,
    data: &mut D,
    _: &mut Self::Aux,
    req: &mut Request<MsgBufferString>,
  ) -> Result<ControlFlow<StatusCode, ()>, E> {
    if let Some(session_state) = data.lease() {
      _trace!(target: crate::_WTX_HTTP_SM, "Connection already has a session");
      if check_expiration(&session_state.expires_at)? {
        _trace!(target: crate::_WTX_HTTP_SM, "Connection session is expired");
        delete_session_cookie(data, req, &self.session_manager, &self.session_store).await?;
        return Ok(ControlFlow::Break(StatusCode::Forbidden));
      }
      return Ok(ControlFlow::Continue(()));
    }
    let mut has_invalid_session = false;
    let mut has_stored_session = true; // `true` because of log-ins
    let mut x_csrf_token_value = None;
    for header in req.msg_data.headers.iter() {
      if data.lease_mut().is_some() && x_csrf_token_value.is_some() {
        break;
      }
      match header.name {
        el if el == <&str>::from(KnownHeaderName::XCsrfToken) => {
          x_csrf_token_value = Some(header.value);
          continue;
        }
        el if el == <&str>::from(KnownHeaderName::Cookie) => {}
        _ => continue,
      }
      let ss_des: SessionState<CS> = {
        let idx = req.msg_data.body.len();
        let cookie_des = CookieStr::parse(header.value, &mut req.msg_data.body)?;
        if cookie_des.generic.name != self.session_manager.inner.0 {
          req.msg_data.body.truncate(idx);
          continue;
        }
        let mut session_guard = self.session_manager.inner.1.lock().await;
        let SessionManagerInner { cookie_def, session_secret, .. } = &mut *session_guard;
        let (name, value) = (cookie_des.generic.name, cookie_des.generic.value);
        let buffer = ArrayVectorU8::<_, { 16 + 28 }>::new();
        let decrypt_rslt = session_secret.peek(&mut buffer.into(), |sp| {
          Aes128GcmGlobal::decrypt_base64_to_buffer(
            name.as_bytes(),
            &mut cookie_def.value,
            value.as_bytes(),
            sp.data().try_into()?,
          )
        });
        req.msg_data.body.truncate(idx);
        let value_json = decrypt_rslt??;
        let json_rslt = serde_json_deserialize_from_slice(value_json);
        cookie_def.value.clear();
        json_rslt?
      };
      _trace!(target: crate::_WTX_HTTP_SM, "A session has been found in headers");
      let Some(ss_db) =
        self.session_store.get_with_unit().await?.lease_mut().read(ss_des.session_key).await?
      else {
        has_stored_session = false;
        break;
      };
      if ss_db.custom_state != ss_des.custom_state {
        has_invalid_session = true;
        break;
      }
      *data.lease_mut() = Some(ss_des);
    }
    // TODO(stable): Polonius
    if has_invalid_session {
      _trace!(target: crate::_WTX_HTTP_SM, "Connection session does not match database ssion");
      delete_session_cookie(data, req, &self.session_manager, &self.session_store).await?;
      return Ok(ControlFlow::Break(StatusCode::Forbidden));
    }
    // TODO(stable): Polonius
    if !has_stored_session {
      _trace!(target: crate::_WTX_HTTP_SM, "Session found in headers does not exist in database");
      delete_session_cookie(data, req, &self.session_manager, &self.session_store).await?;
      return Ok(ControlFlow::Break(StatusCode::Forbidden));
    }
    if let Some(elem) = data.lease_mut() {
      if check_expiration(&elem.expires_at)? {
        _trace!(target: crate::_WTX_HTTP_SM, "Session found in headers is expired");
        delete_session_cookie(data, req, &self.session_manager, &self.session_store).await?;
        return Ok(ControlFlow::Break(StatusCode::Forbidden));
      }
      if req.method.is_mutable() && Some(elem.session_csrf.as_str()) != x_csrf_token_value {
        _trace!(target: crate::_WTX_HTTP_SM, "Session found in headers does not contain a valid CSRF");
        delete_session_cookie(data, req, &self.session_manager, &self.session_store).await?;
        return Ok(ControlFlow::Break(StatusCode::Forbidden));
      }
      _trace!(target: crate::_WTX_HTTP_SM, "Session found in headers has been successfully validated");
    } else {
      let path = req.msg_data.uri.path();
      if self.allowed_paths.iter().all(|el| el != path) {
        _trace!(target: crate::_WTX_HTTP_SM, "Session was not found in headers and path is forbidden");
        delete_session_cookie(data, req, &self.session_manager, &self.session_store).await?;
        return Ok(ControlFlow::Break(StatusCode::Forbidden));
      }
      _trace!(target: crate::_WTX_HTTP_SM, "Session was not found in headers but an allowed path succeeded");
    }
    Ok(ControlFlow::Continue(()))
  }

  #[inline]
  async fn res(
    &self,
    _: &mut D,
    _: &mut Self::Aux,
    _: Response<&mut MsgBufferString>,
  ) -> Result<ControlFlow<StatusCode, ()>, E> {
    Ok(ControlFlow::Continue(()))
  }
}

#[inline]
fn check_expiration(expires_at: &Option<DateTime<Utc>>) -> crate::Result<bool> {
  if let Some(elem) = expires_at
    && *elem < Instant::now_date_time()?.trunc_to_us()
  {
    Ok(true)
  } else {
    Ok(false)
  }
}

#[inline]
async fn delete_session_cookie<CS, D, E, RM>(
  data: &mut D,
  req: &mut Request<MsgBufferString>,
  session_manager: &SessionManager<CS, E>,
  session_store: &SimplePool<RM>,
) -> Result<(), E>
where
  D: LeaseMut<Option<SessionState<CS>>>,
  E: From<crate::Error>,
  RM: ResourceManager<CreateAux = (), Error = E, RecycleAux = ()>,
  RM::Resource: SessionStore<CS, E>,
{
  req.clear();
  let _rslt = session_manager
    .inner
    .1
    .lock()
    .await
    .delete_session_cookie(
      &mut req.msg_data,
      data.lease_mut(),
      &mut ***session_store.get_with_unit().await?,
    )
    .await;
  Ok(())
}
