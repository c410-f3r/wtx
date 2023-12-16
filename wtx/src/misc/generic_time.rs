use core::time::Duration;

/// Tries to support different time machineries of different platforms.
///
/// Currently only supports `std`. For anything else, methods return errors.
#[derive(Clone, Copy, Debug)]
pub struct GenericTime {
  #[cfg(feature = "std")]
  inner: std::time::SystemTime,
}

impl GenericTime {
  /// Returns an instant corresponding to "now".
  #[inline]
  pub fn now() -> crate::Result<Self> {
    #[cfg(feature = "std")]
    {
      Ok(Self { inner: std::time::SystemTime::now() })
    }
    #[cfg(not(feature = "std"))]
    {
      Err(crate::Error::ItIsNotPossibleToUseTimeInNoStd)
    }
  }

  /// Returns the amount of time elapsed from another instant to this one,
  /// or None if that instant is later than this one.
  #[inline]
  pub fn duration_since(&self, _earlier: Self) -> crate::Result<Duration> {
    #[cfg(feature = "std")]
    {
      self.inner.duration_since(_earlier.inner).map_err(|_err| crate::Error::IncorrectHardwareTime)
    }
    #[cfg(not(feature = "std"))]
    {
      Err(crate::Error::ItIsNotPossibleToUseTimeInNoStd)
    }
  }

  /// Returns the amount of time elapsed since this instant was created.
  #[inline]
  pub fn elapsed(&self) -> crate::Result<Duration> {
    Self::now()?.duration_since(*self)
  }

  /// UNIX timestamp of the current time
  #[inline]
  pub fn timestamp(&self) -> crate::Result<Duration> {
    #[cfg(feature = "std")]
    {
      self.duration_since(Self { inner: std::time::UNIX_EPOCH })
    }
    #[cfg(not(feature = "std"))]
    {
      Err(crate::Error::ItIsNotPossibleToUseTimeInNoStd)
    }
  }
}
