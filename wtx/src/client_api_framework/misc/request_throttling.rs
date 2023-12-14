use crate::client_api_framework::misc::{RequestCounter, RequestLimit};

/// A wrapper around [RequestCounter] and [RequestLimit].
#[derive(Clone, Copy, Debug)]
pub struct RequestThrottling {
  /// See [RequestCounter]
  pub rc: RequestCounter,
  /// See [RequestLimit]
  pub rl: RequestLimit,
}

impl RequestThrottling {
  /// Creates an instance with default [RequestCounter] values.
  #[inline]
  pub fn from_rl(rl: RequestLimit) -> crate::Result<Self> {
    Ok(Self { rc: RequestCounter::new()?, rl })
  }
}
