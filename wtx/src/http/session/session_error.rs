/// Session error
#[derive(Debug)]
pub enum SessionError {
  /// Received a session that is expired.
  ExpiredSession,
  /// Received a request which CSRF token differs from the stored CSRF token.
  InvalidCsrfRequest,
  /// Received a session that differs from the stored session.
  InvalidStoredSession,
  /// REceived a session that doesn't exist in the store
  MissingStoredSession,
  /// Path required a session, but there was none
  RequiredSession,
}
