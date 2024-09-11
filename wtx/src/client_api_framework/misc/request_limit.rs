use core::time::Duration;

/// Determines how many times a series of requests can be performed within a certain duration
#[derive(Clone, Copy, Debug)]
pub struct RequestLimit {
  duration: Duration,
  limit: u16,
}

impl RequestLimit {
  /// If `duration` is zero then this structure is basically a no-op. Limits of value `0` will be
  /// converted to `1` to avoid infinite hangs.
  #[inline]
  pub fn new(limit: u16, duration: Duration) -> Self {
    Self { duration, limit: limit.max(1) }
  }

  /// Useful for tests.
  #[inline]
  pub fn unlimited() -> Self {
    Self { duration: Duration::from_secs(0), limit: u16::MAX }
  }

  /// The interval range that can contain a maximum number of [`Self::limit`] requests
  #[inline]
  pub const fn duration(&self) -> &Duration {
    &self.duration
  }

  /// Upper bound or maximum possible number of requests
  #[inline]
  pub fn limit(&self) -> u16 {
    self.limit
  }
}
