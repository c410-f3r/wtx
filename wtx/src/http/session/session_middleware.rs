use crate::{
  calendar::Instant,
  collection::{ArrayVectorU8, Vector},
  crypto::{Aead, Aes128GcmGlobal},
  http::{
    KnownHeaderName, ReqResBuffer, Request, Response, SessionError, SessionManager,
    SessionManagerInner, SessionState, SessionStore, StatusCode, cookie::cookie_str::CookieStr,
    server_framework::Middleware,
  },
  misc::{Lease, LeaseMut, serde_json_deserialize_from_slice},
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

impl<CS, E, RM> SessionMiddleware<CS, E, RM>
where
  E: From<crate::Error>,
  RM: ResourceManager<CreateAux = (), Error = E, RecycleAux = ()>,
  RM::Resource: SessionStore<CS, E>,
{
  #[inline]
  async fn delete_session_cookie<CA>(
    &self,
    ca: &mut CA,
    req: &mut Request<ReqResBuffer>,
  ) -> Result<(), E>
  where
    CA: LeaseMut<Option<SessionState<CS>>>,
  {
    let _rslt = self
      .session_manager
      .inner
      .1
      .lock()
      .await
      .delete_session_cookie(
        &mut req.rrd,
        ca.lease_mut(),
        &mut ***self.session_store.get_with_unit().await?,
      )
      .await;
    Ok(())
  }
}

impl<CA, CS, E, RM, SA> Middleware<CA, E, SA> for SessionMiddleware<CS, E, RM>
where
  CA: LeaseMut<Option<SessionState<CS>>>,
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
    ca: &mut CA,
    _: &mut Self::Aux,
    req: &mut Request<ReqResBuffer>,
    _: &mut SA,
  ) -> Result<ControlFlow<StatusCode, ()>, E> {
    if let Some(session_state) = ca.lease() {
      if let Some(elem) = &session_state.expires_at
        && *elem < Instant::now_date_time(0)?.trunc_to_us()
      {
        self.delete_session_cookie(ca, req).await?;
        return Err(crate::Error::from(SessionError::ExpiredSession).into());
      }
      return Ok(ControlFlow::Continue(()));
    }
    let mut has_stored_session = true; // `true` because of log-ins
    let mut x_csrf_token_value = None;
    for header in req.rrd.headers.iter() {
      if ca.lease_mut().is_some() && x_csrf_token_value.is_some() {
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
        let idx = req.rrd.body.len();
        let cookie_des = CookieStr::parse(header.value, &mut req.rrd.body)?;
        if cookie_des.generic.name != self.session_manager.inner.0 {
          req.rrd.body.truncate(idx);
          continue;
        }
        let mut session_guard = self.session_manager.inner.1.lock().await;
        let SessionManagerInner { cookie_def, session_secret, .. } = &mut *session_guard;
        let (name, value) = (cookie_des.generic.name, cookie_des.generic.value);
        // For some reason a deleted cookie in the frontend only has its contents erased but the cookie still exists.
        if name.is_empty() || value.is_empty() {
          continue;
        }
        let decrypt_rslt = session_secret.peek(&mut ArrayVectorU8::<_, { 16 + 28 }>::new(), |el| {
          Aes128GcmGlobal::decrypt_base64_to_buffer(
            name.as_bytes(),
            &mut cookie_def.value,
            value.as_bytes(),
            el.as_ref().try_into()?,
          )
        });
        req.rrd.body.truncate(idx);
        let value_json = decrypt_rslt??;
        let json_rslt = serde_json_deserialize_from_slice(value_json);
        cookie_def.value.clear();
        json_rslt?
      };
      let ss_db_opt =
        self.session_store.get_with_unit().await?.lease_mut().read(ss_des.session_key).await?;
      let Some(ss_db) = ss_db_opt else {
        has_stored_session = false;
        break;
      };
      if ss_db.custom_state != ss_des.custom_state {
        self.session_store.get_with_unit().await?.lease_mut().delete(&ss_des.session_key).await?;
        return Err(crate::Error::from(SessionError::InvalidStoredSession).into());
      }
      *ca.lease_mut() = Some(ss_des);
    }
    if !has_stored_session {
      req.rrd.clear();
      self.delete_session_cookie(ca, req).await?;
      return Ok(ControlFlow::Break(StatusCode::Forbidden));
    }
    if let Some(local) = ca.lease_mut() {
      if req.method.is_mutable() && Some(local.session_csrf.as_str()) != x_csrf_token_value {
        let session_key = &local.session_key;
        let _rslt = self.session_store.get_with_unit().await?.lease_mut().delete(session_key).await;
        return Err(crate::Error::from(SessionError::InvalidCsrfRequest).into());
      }
    } else {
      let path = req.rrd.uri.path();
      if self.allowed_paths.iter().all(|el| el != path) {
        return Err(crate::Error::from(SessionError::RequiredSession).into());
      }
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
