use core::time::Duration;

/// Tries to support different time machineries of different platforms.
///
/// Currently only supports `std`. For anything else, methods return errors.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct GenericTime {
  #[cfg(feature = "std")]
  _inner: std::time::SystemTime,
  #[cfg(all(feature = "embassy-time", not(any(feature = "std"))))]
  _inner: embassy_time::Instant,
  #[cfg(not(any(feature = "std", feature = "embassy-time")))]
  _inner: (),
}

impl GenericTime {
  /// Returns an instant corresponding to "now".
  #[inline]
  pub fn now() -> Self {
    #[cfg(feature = "std")]
    return Self { _inner: std::time::SystemTime::now() };
    #[cfg(all(feature = "embassy-time", not(any(feature = "std"))))]
    return Self { _inner: embassy_time::Instant::now() };
    #[cfg(not(any(feature = "std", feature = "embassy-time")))]
    return Self { _inner: () };
  }

  /// Returns `Some(t)` where `t` is the time `self + duration` if `t` can be represented as
  /// `GenericTime` (which means it's inside the bounds of the underlying data structure), `None`
  /// otherwise.
  #[inline]
  pub fn checked_add(&self, _duration: Duration) -> crate::Result<Self> {
    #[cfg(feature = "std")]
    return Ok(Self {
      _inner: self._inner.checked_add(_duration).ok_or(crate::Error::InvalidTimeArithmetic)?,
    });
    #[cfg(all(feature = "embassy-time", not(any(feature = "std"))))]
    return Ok(Self {
      _inner: self
        ._inner
        .checked_add(embassy_time::Duration::from_secs(_duration.as_secs()))
        .ok_or(crate::Error::InvalidTimeArithmetic)?,
    });
    #[cfg(not(any(feature = "std", feature = "embassy-time")))]
    return Err(crate::Error::GenericTimeNeedsBackend);
  }

  /// Returns `Some(t)` where `t` is the time `self - duration` if `t` can be represented as
  /// `GenericTime` (which means it's inside the bounds of the underlying data structure), `None`
  /// otherwise.
  #[inline]
  pub fn checked_sub(&self, _duration: Duration) -> crate::Result<Self> {
    #[cfg(feature = "std")]
    return Ok(Self {
      _inner: self._inner.checked_sub(_duration).ok_or(crate::Error::InvalidTimeArithmetic)?,
    });
    #[cfg(all(feature = "embassy-time", not(any(feature = "std"))))]
    return Ok(Self {
      _inner: self
        ._inner
        .checked_sub(embassy_time::Duration::from_secs(_duration.as_secs()))
        .ok_or(crate::Error::InvalidTimeArithmetic)?,
    });
    #[cfg(not(any(feature = "std", feature = "embassy-time")))]
    return Err(crate::Error::GenericTimeNeedsBackend);
  }

  /// Returns the amount of time elapsed from another instant to this one,
  /// or None if that instant is later than this one.
  #[inline]
  pub fn duration_since(&self, _earlier: Self) -> crate::Result<Duration> {
    #[cfg(feature = "std")]
    return self
      ._inner
      .duration_since(_earlier._inner)
      .map_err(|_err| crate::Error::InvalidHardwareTime);
    #[cfg(all(feature = "embassy-time", not(any(feature = "std"))))]
    return Ok(Duration::from_micros(
      self
        ._inner
        .checked_duration_since(_earlier._inner)
        .ok_or(crate::Error::InvalidTimeArithmetic)?
        .as_micros(),
    ));
    #[cfg(not(any(feature = "std", feature = "embassy-time")))]
    return Err(crate::Error::GenericTimeNeedsBackend);
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
    return Self::now().duration_since(Self { _inner: std::time::UNIX_EPOCH });
    #[cfg(all(feature = "embassy-time", not(any(feature = "std"))))]
    return Self::now().duration_since(Self { _inner: embassy_time::Instant::from_secs(0) });
    #[cfg(not(any(feature = "std", feature = "embassy-time")))]
    return Err(crate::Error::GenericTimeNeedsBackend);
  }
}

/// Provides the current time
#[derive(Debug)]
pub struct GenericTimeProvider;

#[cfg(feature = "rustls")]
impl rustls::time_provider::TimeProvider for GenericTimeProvider {
  #[inline]
  fn current_time(&self) -> Option<rustls_pki_types::UnixTime> {
    Some(rustls_pki_types::UnixTime::since_unix_epoch(GenericTime::timestamp().ok()?))
  }
}
