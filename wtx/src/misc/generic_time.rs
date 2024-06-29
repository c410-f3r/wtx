use core::time::Duration;

/// Tries to support different time machineries of different platforms.
///
/// Currently only supports `std`. For anything else, methods return errors.
#[derive(Clone, Copy, Debug)]
pub struct GenericTime {
  #[cfg(feature = "std")]
  inner: std::time::SystemTime,
  #[cfg(all(feature = "embassy-time", not(any(feature = "std"))))]
  inner: embassy_time::Instant,
  #[cfg(not(any(feature = "std", feature = "embassy-time")))]
  _inner: (),
}

impl GenericTime {
  /// Returns an instant corresponding to "now".
  #[inline]
  pub fn now() -> Self {
    #[cfg(feature = "std")]
    {
      Self { inner: std::time::SystemTime::now() }
    }
    #[cfg(all(feature = "embassy-time", not(any(feature = "std"))))]
    {
      Self { inner: embassy_time::Instant::now() }
    }
    #[cfg(not(any(feature = "std", feature = "embassy-time")))]
    Self { _inner: () }
  }

  /// Returns the amount of time elapsed from another instant to this one,
  /// or None if that instant is later than this one.
  #[inline]
  pub fn duration_since(&self, _earlier: Self) -> crate::Result<Duration> {
    #[cfg(feature = "std")]
    {
      self
        .inner
        .duration_since(_earlier.inner)
        .map_err(|_err| crate::Error::MISC_InvalidHardwareTime)
    }
    #[cfg(all(feature = "embassy-time", not(any(feature = "std"))))]
    {
      Ok(Duration::from_micros(
        self
          .inner
          .checked_duration_since(_earlier.inner)
          .ok_or(crate::Error::MISC_InvalidHardwareTime)?
          .as_micros(),
      ))
    }
    #[cfg(not(any(feature = "std", feature = "embassy-time")))]
    {
      Err(crate::Error::MISC_GenericTimeNeedsBackend)
    }
  }

  /// Returns the amount of time elapsed since this instant was created.
  #[inline]
  pub fn elapsed(&self) -> crate::Result<Duration> {
    Self::now().duration_since(*self)
  }

  /// UNIX timestamp of the current time
  #[inline]
  pub fn timestamp() -> crate::Result<Duration> {
    #[cfg(feature = "std")]
    {
      Self::now().duration_since(Self { inner: std::time::UNIX_EPOCH })
    }
    #[cfg(all(feature = "embassy-time", not(any(feature = "std"))))]
    {
      Self::now().duration_since(Self { inner: embassy_time::Instant::from_micros(0) })
    }
    #[cfg(not(any(feature = "std", feature = "embassy-time")))]
    {
      Err(crate::Error::MISC_GenericTimeNeedsBackend)
    }
  }
}
