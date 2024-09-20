use core::time::Duration;

/// Tries to support different time machineries of different platforms.
///
/// Currently only supports `std`. For anything else, methods return errors.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct GenericTime {
  #[cfg(feature = "std")]
  inner: std::time::SystemTime,
  #[cfg(not(feature = "std"))]
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
    #[cfg(not(feature = "std"))]
    Self { _inner: () }
  }

  /// Returns `Some(t)` where `t` is the time `self + duration` if `t` can be represented as
  /// `GenericTime` (which means it's inside the bounds of the underlying data structure), `None`
  /// otherwise.
  #[inline]
  pub fn checked_add(&self, _duration: Duration) -> crate::Result<Self> {
    #[cfg(feature = "std")]
    {
      Ok(Self {
        inner: self.inner.checked_add(_duration).ok_or(crate::Error::InvalidHardwareTime)?,
      })
    }
    #[cfg(not(feature = "std"))]
    {
      Err(crate::Error::GenericTimeNeedsBackend)
    }
  }

  /// Returns `Some(t)` where `t` is the time `self - duration` if `t` can be represented as
  /// `GenericTime` (which means it's inside the bounds of the underlying data structure), `None`
  /// otherwise.
  #[inline]
  pub fn checked_sub(&self, _duration: Duration) -> crate::Result<Self> {
    #[cfg(feature = "std")]
    {
      Ok(Self {
        inner: self.inner.checked_sub(_duration).ok_or(crate::Error::InvalidHardwareTime)?,
      })
    }
    #[cfg(not(feature = "std"))]
    {
      Err(crate::Error::GenericTimeNeedsBackend)
    }
  }

  /// Returns the amount of time elapsed from another instant to this one,
  /// or None if that instant is later than this one.
  #[inline]
  pub fn duration_since(&self, _earlier: Self) -> crate::Result<Duration> {
    #[cfg(feature = "std")]
    {
      self.inner.duration_since(_earlier.inner).map_err(|_err| crate::Error::InvalidHardwareTime)
    }
    #[cfg(not(feature = "std"))]
    {
      Err(crate::Error::GenericTimeNeedsBackend)
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
    #[cfg(not(feature = "std"))]
    {
      Err(crate::Error::GenericTimeNeedsBackend)
    }
  }
}
