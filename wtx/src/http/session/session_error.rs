/// Session error
#[derive(Debug)]
pub enum SessionError {
  /// Received a session that is expired.
  ExpiredSession,
  /// Received a request which CSRF token differs from the stored CSRF token.
  InvalidCsrfRequest,
  /// The secret must have a specific number of bytes.
  InvalidSecretLength,
  /// Received a session that differs from the stored session.
  InvalidStoredSession,
  /// Path required a session, but there was none
  RequiredSession,
}
