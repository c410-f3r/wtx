use crate::{
  calendar::Instant,
  collection::Vector,
  crypto::{Aead, Aes256GcmAesGcm},
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
    let mut session_guard = self.session_manager.inner.lock().await;
    let SessionManagerInner { cookie_def, session_secret, .. } = &mut *session_guard;
    if let Some(elem) = ca.lease() {
      if let Some(expires) = &elem.expires_at
        && expires >= &Instant::now_date_time(0)?.trunc_to_us()
      {
        let _rslt =
          self.session_store.get_with_unit().await?.lease_mut().delete(&elem.session_key).await;
        return Err(crate::Error::from(SessionError::ExpiredSession).into());
      }
      return Ok(ControlFlow::Continue(()));
    }
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
        if cookie_des.generic.name != cookie_def.name {
          continue;
        }
        let (name, value) = (cookie_des.generic.name, cookie_des.generic.value);
        let decrypt_rslt = Aes256GcmAesGcm::decrypt_base64(
          name.as_bytes(),
          &mut cookie_def.value,
          value.as_bytes(),
          session_secret.data()?,
        );
        req.rrd.body.truncate(idx);
        let value_json = decrypt_rslt?;
        let json_rslt = serde_json_deserialize_from_slice(value_json);
        cookie_def.value.clear();
        json_rslt.map_err(Into::into)?
      };
      let ss_db_opt = {
        let mut lock = self.session_store.get(&(), &()).await?;
        lock.lease_mut().read(ss_des.session_key).await?
      };
      let Some(ss_db) = ss_db_opt else {
        return Err(crate::Error::from(SessionError::MissingStoredSession).into());
      };
      if ss_db.custom_state != ss_des.custom_state {
        self.session_store.get(&(), &()).await?.lease_mut().delete(&ss_des.session_key).await?;
        return Err(crate::Error::from(SessionError::InvalidStoredSession).into());
      }
      *ca.lease_mut() = Some(ss_des);
    }
    if let Some(local) = ca.lease_mut() {
      if req.method.is_mutable() && Some(local.session_csrf.as_str()) != x_csrf_token_value {
        let session_key = &local.session_key;
        let _rslt = self.session_store.get(&(), &()).await?.lease_mut().delete(session_key).await;
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
