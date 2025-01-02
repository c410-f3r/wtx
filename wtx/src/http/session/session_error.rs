/// Session error
#[derive(Debug)]
pub enum SessionError {
  /// Received a session that is expired.
  ExpiredSession,
  /// REceived a session that differs from the stored session.
  InvalidStoredSession,
  /// REceived a session that doesn't exist in the store
  MissingStoredSession,
  /// Path required a session, but there was none
  RequiredSession,
}
