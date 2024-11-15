mod session_builder;
mod session_decoder;
mod session_enforcer;
mod session_error;
mod session_manager;
mod session_state;
mod session_store;

use crate::http::server_framework::ConnAux;
pub use session_builder::SessionBuilder;
pub use session_decoder::SessionDecoder;
pub use session_enforcer::SessionEnforcer;
pub use session_error::SessionError;
pub use session_manager::{SessionManager, SessionManagerInner};
pub use session_state::SessionState;
pub use session_store::SessionStore;

type SessionId = [u8; 16];
type SessionKey = [u8; 32];
/// [`Session`] backed by `tokio`
#[cfg(feature = "tokio")]
pub type SessionTokio<CS, E, S> =
  Session<crate::misc::Arc<tokio::sync::Mutex<SessionManagerInner<CS, E>>>, S>;

/// Allows the management of state across requests within a connection.
#[derive(Clone, Debug)]
pub struct Session<I, S> {
  /// Manager
  pub manager: SessionManager<I>,
  /// Store
  pub store: S,
}

impl<I, S> Session<I, S> {
  /// Allows the specification of custom parameters.
  #[inline]
  pub fn builder(store: S) -> SessionBuilder<S> {
    SessionBuilder::new(store)
  }
}

impl<I, S> ConnAux for Session<I, S> {
  type Init = Self;

  #[inline]
  fn conn_aux(init: Self::Init) -> crate::Result<Self> {
    Ok(init)
  }
}
