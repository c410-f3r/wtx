use crate::http::session::SessionId;
use chrono::{DateTime, Utc};

/// Data that is saved in the corresponding store.
#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SessionState<CS> {
  /// Custom state
  pub custom_state: CS,
  /// Cookie expiration
  pub expire: Option<DateTime<Utc>>,
  /// Identifier
  pub id: SessionId,
}
