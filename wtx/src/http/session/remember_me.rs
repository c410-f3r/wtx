/// Makes session cookies persistent.
///
/// When configured in [`SessionManagerBuilder`](crate::http::SessionManagerBuilder), session cookies
/// are emitted with an `Expires` attribute and therefore survive browser restarts. Without it, the
/// cookie is session-only and is discarded when the browser closes.
#[derive(Debug, Default)]
pub struct RememberMe {}
