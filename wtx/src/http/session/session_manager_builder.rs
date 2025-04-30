use crate::{
  collection::{ArrayString, Vector},
  http::{
    SessionManager, SessionStore,
    cookie::{SameSite, cookie_generic::CookieGeneric},
    session::{SessionManagerInner, SessionSecret},
  },
  misc::{Lock, sleep},
  rng::CryptoRng,
};
use chrono::{DateTime, Utc};
use core::{marker::PhantomData, time::Duration};

/// Default and optional parameters for the construction of a [`SessionManager`].
#[derive(Debug)]
pub struct SessionManagerBuilder {
  pub(crate) cookie_def: CookieGeneric<&'static str, Vector<u8>>,
  pub(crate) inspection_interval: Duration,
}

impl SessionManagerBuilder {
  #[inline]
  pub(crate) const fn new() -> Self {
    Self {
      cookie_def: CookieGeneric {
        domain: "",
        expires: None,
        http_only: true,
        max_age: None,
        name: "id",
        path: "/",
        same_site: Some(SameSite::Strict),
        secure: true,
        value: Vector::new(),
      },
      inspection_interval: Duration::from_secs(60 * 30),
    }
  }

  /// Creates a new [`SessionManager`] with random generated keys. It is up to the caller to
  /// provide a good RNG.
  ///
  /// The returned [`Future`] is responsible for deleting expired sessions at an interval defined by
  /// [`Self::inspection_interval`] and should be called in a separated task.
  ///
  /// If the backing store already has a system that automatically removes outdated sessions like
  /// SQL triggers, then the [`Future`] can be ignored.
  #[inline]
  pub fn build_generating_key<CS, E, RNG, SMI, SS>(
    self,
    rng: &mut RNG,
    session_store: SS,
  ) -> crate::Result<(
    impl Future<Output = Result<(), E>> + use<CS, E, RNG, SMI, SS>,
    SessionManager<SMI>,
  )>
  where
    E: From<crate::Error>,
    RNG: CryptoRng,
    SMI: Lock<Resource = SessionManagerInner<CS, E>>,
    SS: Clone + SessionStore<CS, E>,
  {
    Ok(Self::build_with_key(
      self,
      ArrayString::from_iter(rng.ascii_graphic_iter().take(32))?,
      session_store,
    ))
  }

  /// Creates a new [`SessionManager`] with the provided key. It is up to the caller to
  /// provide a cryptographically secure secret.
  ///
  /// The returned [`Future`] is responsible for deleting expired sessions at an interval defined by
  /// [`Self::inspection_interval`] and should be called in a separated task.
  ///
  /// If the backing store already has a system that automatically removes outdated sessions like
  /// SQL triggers, then the [`Future`] can be ignored.
  #[inline]
  pub fn build_with_key<CS, E, SMI, SS>(
    self,
    session_secret: SessionSecret,
    mut session_store: SS,
  ) -> (impl Future<Output = Result<(), E>>, SessionManager<SMI>)
  where
    E: From<crate::Error>,
    SMI: Lock<Resource = SessionManagerInner<CS, E>>,
    SS: Clone + SessionStore<CS, E>,
  {
    let Self { cookie_def, inspection_interval } = self;
    (
      async move {
        loop {
          session_store.delete_expired().await?;
          sleep(inspection_interval).await?;
        }
      },
      SessionManager {
        inner: SMI::new(SessionManagerInner { cookie_def, phantom: PhantomData, session_secret }),
      },
    )
  }

  /// Defines the host to which the cookie will be sent.
  #[inline]
  pub const fn domain(mut self, elem: &'static str) -> Self {
    self.cookie_def.domain = elem;
    self
  }

  /// Indicates the maximum lifetime of the cookie as an HTTP-date timestamp.
  ///
  /// If [Self::max_age] is set, then this parameter is ignored when setting the cookie in the
  /// header.
  #[inline]
  pub fn expires(mut self, elem: Option<DateTime<Utc>>) -> Self {
    self.cookie_def.expires = elem;
    self
  }

  /// Forbids JavaScript from accessing the cookie.
  #[inline]
  pub fn http_only(mut self, elem: bool) -> Self {
    self.cookie_def.http_only = elem;
    self
  }

  /// The amount of time the future returned by the building methods will wait before
  /// deleting expired sessions.
  #[inline]
  pub fn inspection_interval(mut self, elem: Duration) -> Self {
    self.inspection_interval = elem;
    self
  }

  /// Cookie name.
  #[inline]
  pub fn name(mut self, elem: &'static str) -> Self {
    self.cookie_def.name = elem;
    self
  }

  /// Indicates the number of seconds until the cookie expires.
  #[inline]
  pub fn max_age(mut self, elem: Option<Duration>) -> Self {
    self.cookie_def.max_age = elem;
    self
  }

  /// Indicates the path that must exist in the requested URL for the browser to send the Cookie
  /// header.
  #[inline]
  pub fn path(mut self, elem: &'static str) -> Self {
    self.cookie_def.path = elem;
    self
  }

  /// Controls whether or not a cookie is sent with cross-site requests.
  #[inline]
  pub fn same_site(mut self, elem: Option<SameSite>) -> Self {
    self.cookie_def.same_site = elem;
    self
  }

  /// Indicates that the cookie is sent to the server only when a request is made with the `https`
  /// scheme.
  #[inline]
  pub fn secure(mut self, elem: bool) -> Self {
    self.cookie_def.secure = elem;
    self
  }
}
