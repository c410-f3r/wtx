/// Session error
#[derive(Debug)]
pub enum SessionError {
  /// Path required a session, but there was none
  RequiredSessionIsNotPresent,
}
