use crate::{
  collections::{ShortBoxStrU8, VectorU16},
  futures::Sleep,
  http::{
    DEFAULT_SESSION_NAME, RememberMe, SameSite, SessionError, SessionManager, SessionManagerInner,
    SessionStore, cookie::SetCookieGeneric, session::SessionManagerSync,
  },
  rng::CryptoRng,
  secret::SecretArray,
  sync::{Arc, AsyncMutex},
};
use core::time::Duration;

/// Default and optional parameters for the construction of a [`SessionManager`].
///
/// # Rotation
///
/// - **Lifetime**: Once half of the lifetime has elapsed, the session key is rotated and its
///   expiration is extended by another full lifetime.
/// - **Lifetime upper bound**: Once reached, rotation no longer extends lifetime and it also
///   expires the session.
/// - **Grace period**: The previous session key remains valid for this amount of time after a
///   rotation, so requests that were already in flight do not lose their session.
#[derive(Debug)]
pub struct SessionManagerBuilder {
  cookie_def: SetCookieGeneric<ShortBoxStrU8, VectorU16<u8>>,
  grace_period: Duration,
  inspection_interval: Duration,
  lifetime: Option<Duration>,
  lifetime_ub: Option<Duration>,
  remember_me: Option<RememberMe>,
}

impl SessionManagerBuilder {
  /// Default grace period.
  pub const DEFAULT_GRACE_PERIOD: Duration = Duration::from_secs(30);
  /// Default inspection interval.
  pub const DEFAULT_INSPECTION_INTERVAL: Duration = Duration::from_mins(30);

  pub(crate) fn new() -> Self {
    Self {
      cookie_def: SetCookieGeneric {
        domain: ShortBoxStrU8::default(),
        expires: None,
        http_only: true,
        max_age: None,
        name: DEFAULT_SESSION_NAME.try_into().unwrap_or_default(),
        path: "/".try_into().unwrap_or_default(),
        same_site: Some(SameSite::Strict),
        secure: true,
        value: VectorU16::new(),
      },
      grace_period: Self::DEFAULT_GRACE_PERIOD,
      inspection_interval: Self::DEFAULT_INSPECTION_INTERVAL,
      lifetime: None,
      lifetime_ub: None,
      remember_me: None,
    }
  }

  /// Creates a new [`SessionManager`] with random generated keys.
  ///
  /// The returned [`Future`] is responsible for deleting expired sessions at an interval defined by
  /// [`Self::inspection_interval`] and should be called in a separated task.
  ///
  /// If the backing store already has a system that automatically removes outdated sessions like
  /// SQL triggers, then the [`Future`] can be ignored.
  #[inline]
  pub fn build_generating_key<CS, E, RNG, SS>(
    self,
    mut rng: RNG,
    session_store: SS,
  ) -> crate::Result<(impl Future<Output = Result<(), E>>, SessionManager<CS, E, RNG>)>
  where
    E: From<crate::Error>,
    RNG: CryptoRng,
    SS: SessionStore<CS, E>,
  {
    let mut data = [0u8; 16];
    rng.fill_slice(&mut data);
    let session_secret = SecretArray::new(&mut data)?;
    Self::build_with_key(self, rng, session_secret, session_store)
  }

  /// Creates a new [`SessionManager`] with the provided key. It is up to the caller to
  /// provide a 16 bytes cryptographically secure secret.
  ///
  /// The returned [`Future`] is responsible for deleting expired sessions at an interval defined by
  /// [`Self::inspection_interval`] and should be called in a separated task.
  ///
  /// If the backing store already has a system that automatically removes outdated sessions like
  /// SQL triggers, then the [`Future`] can be ignored.
  #[inline]
  pub fn build_with_key<CS, E, RNG, SS>(
    self,
    rng: RNG,
    session_secret: SecretArray<16>,
    mut session_store: SS,
  ) -> crate::Result<(impl Future<Output = Result<(), E>>, SessionManager<CS, E, RNG>)>
  where
    E: From<crate::Error>,
    RNG: CryptoRng,
    SS: SessionStore<CS, E>,
  {
    let Self {
      mut cookie_def,
      grace_period,
      inspection_interval,
      lifetime,
      lifetime_ub,
      remember_me: _,
    } = self;
    cookie_def.expires = None;
    cookie_def.max_age = None;
    let shortest_lifetime_ub =
      lifetime_ub.unwrap_or(SessionManager::<CS, E, RNG>::DEFAULT_LF_UB_EPHEMERAL);
    let shortest_lifetime = lifetime
      .unwrap_or(SessionManager::<CS, E, RNG>::DEFAULT_LF_EPHEMERAL)
      .min(shortest_lifetime_ub);
    if grace_period.as_secs() >= shortest_lifetime.as_secs().div_ceil(2) {
      return Err(SessionError::InvalidGracePeriod.into());
    }
    Ok((
      async move {
        loop {
          session_store.delete_expired().await?;
          Sleep::new(inspection_interval)?.await?;
        }
      },
      SessionManager {
        inner: Arc::new(SessionManagerInner::new(
          grace_period,
          lifetime,
          #[cfg(feature = "http2-server-framework")]
          cookie_def.name,
          AsyncMutex::new(SessionManagerSync::new(cookie_def, lifetime_ub, rng, session_secret)),
        )),
      },
    ))
  }

  /// Defines the host to which the cookie will be sent.
  #[inline]
  #[must_use = must_use_itself_generic_doc!()]
  pub fn domain(mut self, elem: ShortBoxStrU8) -> Self {
    self.cookie_def.domain = elem;
    self
  }

  /// Amount of time that a rotated session key remains valid.
  #[inline]
  #[must_use = must_use_itself_generic_doc!()]
  pub const fn grace_period(mut self, elem: Duration) -> Self {
    self.grace_period = elem;
    self
  }

  /// Forbids `JavaScript` from accessing the cookie.
  #[inline]
  #[must_use = must_use_itself_generic_doc!()]
  pub const fn http_only(mut self, elem: bool) -> Self {
    self.cookie_def.http_only = elem;
    self
  }

  /// The amount of time the future returned by the building methods will wait before deleting
  /// expired sessions.
  #[inline]
  #[must_use = must_use_itself_generic_doc!()]
  pub const fn inspection_interval(mut self, elem: Duration) -> Self {
    self.inspection_interval = elem;
    self
  }

  /// Lifetime.
  #[inline]
  #[must_use = must_use_itself_generic_doc!()]
  pub const fn lifetime(mut self, elem: Duration) -> Self {
    self.lifetime = Some(elem);
    self
  }

  /// Upper bound for the lifetime.
  #[inline]
  #[must_use = must_use_itself_generic_doc!()]
  pub const fn lifetime_ub(mut self, elem: Duration) -> Self {
    self.lifetime_ub = Some(elem);
    self
  }

  /// Cookie name.
  #[inline]
  pub fn name(mut self, elem: &str) -> crate::Result<Self> {
    self.cookie_def.name = elem.try_into()?;
    Ok(self)
  }

  /// Indicates the path that must exist in the requested URL for the browser to send the Cookie
  /// header.
  #[inline]
  #[must_use = must_use_itself_generic_doc!()]
  pub fn path(mut self, elem: ShortBoxStrU8) -> Self {
    self.cookie_def.path = elem;
    self
  }

  /// Uses a custom set of parameters for long-lived sessions.
  #[inline]
  #[must_use = must_use_itself_generic_doc!()]
  pub fn remember_me(mut self, elem: RememberMe) -> Self {
    self.remember_me = Some(elem);
    self
  }

  /// Controls whether or not a cookie is sent with cross-site requests.
  #[inline]
  #[must_use = must_use_itself_generic_doc!()]
  pub const fn same_site(mut self, elem: Option<SameSite>) -> Self {
    self.cookie_def.same_site = elem;
    self
  }

  /// Indicates that the cookie is sent to the server only when a request is made with the `https`
  /// scheme.
  #[inline]
  #[must_use = must_use_itself_generic_doc!()]
  pub const fn secure(mut self, elem: bool) -> Self {
    self.cookie_def.secure = elem;
    self
  }
}
