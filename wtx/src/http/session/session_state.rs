use crate::{
  calendar::{DateTime, Utc},
  http::session::{SessionCsrf, SessionKey},
};

/// Data that is saved in the corresponding store.
#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SessionState<CS> {
  /// Custom state
  pub custom_state: CS,
  /// Cookie expiration
  pub expires_at: Option<DateTime<Utc>>,
  /// CSRF token
  pub session_csrf: SessionCsrf,
  /// Identifier
  pub session_key: SessionKey,
}

impl<CS> SessionState<CS> {
  /// Constructor shortcut
  #[inline]
  pub const fn new(
    custom_state: CS,
    expires_at: Option<DateTime<Utc>>,
    session_csrf: SessionCsrf,
    session_key: SessionKey,
  ) -> Self {
    Self { custom_state, expires_at, session_csrf, session_key }
  }
}
