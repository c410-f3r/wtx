use crate::http::session::{SessionCsrf, SessionKey};

/// Data that is saved in the corresponding store.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SessionState<CS> {
  /// Custom state
  pub custom_state: CS,
  /// UNIX timestamp, in seconds, where the session stops being valid.
  pub expires_at: u64,
  /// No session can be greater than this upper bound, specified as a UNIX timestamp in seconds
  pub expires_at_ub: u64,
  /// Whether the session cookie should survive browser restarts.
  pub remember_me: bool,
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
    expires_at: u64,
    expires_at_ub: u64,
    remember_me: bool,
    session_csrf: SessionCsrf,
    session_key: SessionKey,
  ) -> Self {
    Self { custom_state, expires_at, expires_at_ub, remember_me, session_csrf, session_key }
  }
}
