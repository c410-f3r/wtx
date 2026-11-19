mod remember_me;
#[cfg(feature = "database")]
mod session_custom_state;
mod session_error;
mod session_manager;
mod session_manager_builder;
#[cfg(feature = "http2-server-framework")]
mod session_middleware;
mod session_state;
mod session_store;

pub use remember_me::RememberMe;
#[cfg(feature = "database")]
pub use session_custom_state::SessionCustomState;
pub use session_error::SessionError;
pub use session_manager::*;
pub use session_manager_builder::SessionManagerBuilder;
#[cfg(feature = "http2-server-framework")]
pub use session_middleware::SessionMiddleware;
pub use session_state::SessionState;
pub use session_store::SessionStore;

/// Default session cookie name.
pub const DEFAULT_SESSION_NAME: &str = "wtx-sid";

/// Cross-site request forgery token
pub type SessionCsrf = crate::collections::FixedStr<32>;
/// Session key token
pub type SessionKey = crate::collections::FixedStr<32>;

/// Convert an optional [`SessionState`] a `Result`.
#[inline]
pub fn session_state_rslt<CS>(ss: &Option<SessionState<CS>>) -> crate::Result<&SessionState<CS>> {
  Ok(ss.as_ref().ok_or(SessionError::RequiredSessionIsNotPresent)?)
}
