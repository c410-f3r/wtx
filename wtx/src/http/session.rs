mod session_error;
mod session_manager;
mod session_manager_builder;
#[cfg(feature = "http-server-framework")]
mod session_middleware;
mod session_state;
mod session_store;

pub use session_error::SessionError;
pub use session_manager::*;
pub use session_manager_builder::SessionManagerBuilder;
#[cfg(feature = "http-server-framework")]
pub use session_middleware::SessionMiddleware;
pub use session_state::SessionState;
pub use session_store::SessionStore;

type SessionCsrf = crate::collection::ArrayStringU8<32>;
type SessionKey = crate::collection::ArrayStringU8<32>;
type SessionSecret = crate::collection::ArrayStringU8<32>;
