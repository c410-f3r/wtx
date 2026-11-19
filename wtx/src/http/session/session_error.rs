/// Session error
#[derive(Debug)]
pub enum SessionError {
  /// The grace period must be shorter than half of the session lifetime.
  InvalidGracePeriod,
  /// Path required a session, but there was none
  RequiredSessionIsNotPresent,
}
