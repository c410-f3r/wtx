use crate::{
  calendar::Instant,
  collection::Vector,
  http::{
    KnownHeaderName, ReqResBuffer, Request, Response, SessionError, SessionManager,
    SessionManagerInner, SessionState, SessionStore, StatusCode,
    cookie::{cookie_str::CookieStr, decrypt_cookie},
    server_framework::Middleware,
  },
  misc::{Lease, LeaseMut},
  pool::{Pool, ResourceManager},
  sync::Lock,
};
use alloc::string::String;
use core::ops::ControlFlow;
use serde::de::DeserializeOwned;

/// Decodes cookies received from requests and manages them.
#[derive(Debug)]
pub struct SessionMiddleware<SMI, SS> {
  allowed_paths: Vector<String>,
  session_manager: SessionManager<SMI>,
  session_store: SS,
}

impl<SMI, SS> SessionMiddleware<SMI, SS> {
  /// New instance
  #[inline]
  pub fn new(
    allowed_paths: Vector<String>,
    session_manager: SessionManager<SMI>,
    session_store: SS,
  ) -> Self {
    Self { allowed_paths, session_manager, session_store }
  }
}

impl<CA, CS, E, RM, SMI, SS, SA> Middleware<CA, E, SA> for SessionMiddleware<SMI, SS>
where
  CA: LeaseMut<Option<SessionState<CS>>>,
  CS: DeserializeOwned + PartialEq,
  E: From<crate::Error>,
  RM: ResourceManager<CreateAux = (), Error = E, RecycleAux = ()>,
  RM::Resource: SessionStore<CS, E>,
  SMI: Lock<Resource = SessionManagerInner<CS, E>>,
  SS: Pool<ResourceManager = RM>,
  for<'any> SS::GetElem<'any>: LeaseMut<RM::Resource>,
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
          self.session_store.get(&(), &()).await?.lease_mut().delete(&elem.session_key).await;
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
        let decrypt_rslt = decrypt_cookie(
          &mut cookie_def.value,
          session_secret.data()?,
          (name.as_bytes(), value.as_bytes()),
        );
        req.rrd.body.truncate(idx);
        let value_json = decrypt_rslt?;
        let json_rslt = serde_json::from_slice(value_json);
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
      let is_same = Some(local.session_csrf.as_ref()) != x_csrf_token_value.map(|el| el.as_bytes());
      if req.method.is_mutable() && is_same {
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
