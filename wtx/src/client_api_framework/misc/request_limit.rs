use core::{num::NonZeroU16, time::Duration};

/// Determines how many times a series of requests can be performed within a certain duration
#[derive(Clone, Copy, Debug)]
pub struct RequestLimit {
  duration: Duration,
  limit: NonZeroU16,
}

impl RequestLimit {
  /// If `duration` is zero then this structure is basically a no-op.
  ///
  /// `limit` must start at 1 to always exist at least one request.
  #[inline]
  pub fn new(limit: u16, duration: Duration) -> crate::Result<Self> {
    Ok(Self { duration, limit: limit.try_into()? })
  }

  /// Useful for tests.
  #[inline]
  pub fn unlimited() -> Self {
    Self { duration: Duration::from_secs(0), limit: NonZeroU16::MAX }
  }

  /// The interval range that can contain a maximum number of [`Self::limit`] requests
  #[inline]
  pub const fn duration(&self) -> &Duration {
    &self.duration
  }

  /// Upper bound or maximum possible number of requests
  #[inline]
  pub fn limit(&self) -> u16 {
    self.limit.into()
  }
}

#[test]
fn limit_can_not_be_zero() {
  assert!(RequestLimit::new(0, Duration::default()).is_err())
}
