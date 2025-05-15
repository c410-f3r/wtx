use crate::calendar::CalendarError;
use core::time::Duration;

/// Time provider suitable for operations involving intervals.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Instant {
  #[cfg(feature = "std")]
  _inner: std::time::SystemTime,
  #[cfg(all(feature = "embassy-time", not(any(feature = "std"))))]
  _inner: embassy_time::Instant,
  #[cfg(not(any(feature = "std", feature = "embassy-time")))]
  _inner: (),
}

impl Instant {
  /// Creates an instance that refers the UNIX epoch (1970-01-01).
  ///
  #[doc = doc_epoch!()]
  #[inline]
  pub const fn epoch(_secs: u64) -> crate::Result<Self> {
    #[cfg(feature = "std")]
    return Ok(Self { _inner: std::time::UNIX_EPOCH });
    #[cfg(all(feature = "embassy-time", not(any(feature = "std"))))]
    return Ok(Self { _inner: embassy_time::Instant::from_secs(_secs) });
    #[cfg(not(any(feature = "std", feature = "embassy-time")))]
    return Err(crate::Error::CalendarError(CalendarError::InstantNeedsBackend));
  }

  /// Returns the system time corresponding to "now". Can have different durations
  /// depending on the underlying provider.
  #[inline]
  pub fn now() -> Self {
    #[cfg(feature = "std")]
    return Self { _inner: std::time::SystemTime::now() };
    #[cfg(all(feature = "embassy-time", not(any(feature = "std"))))]
    return Self { _inner: embassy_time::Instant::now() };
    #[cfg(not(any(feature = "std", feature = "embassy-time")))]
    return Self { _inner: () };
  }

  /// Returns a new `DateTime` instance with the current date and time in UTC based on the
  /// UNIX epoch.
  ///
  #[cfg(feature = "calendar")]
  #[doc = doc_epoch!()]
  #[inline]
  pub fn now_date_time(secs: u64) -> crate::Result<crate::calendar::DateTime> {
    let timestamp = Instant::now_timestamp(secs)?;
    crate::calendar::DateTime::from_timestamp_secs_and_ns(
      timestamp.as_secs().cast_signed(),
      timestamp.subsec_nanos().try_into()?,
    )
  }

  /// Constructor that returns the number of non-leap seconds since the UNIX epoch.
  ///
  #[doc = doc_epoch!()]
  #[inline]
  pub fn now_timestamp(secs: u64) -> crate::Result<Duration> {
    Self::now().duration_since(Self::epoch(secs)?)
  }

  /// Returns the addition if the resulting value is within bounds.
  #[inline]
  pub fn add(&self, _duration: Duration) -> crate::Result<Self> {
    #[cfg(feature = "std")]
    return Ok(Self {
      _inner: self._inner.checked_add(_duration).ok_or(CalendarError::ArithmeticOverflow)?,
    });
    #[cfg(all(feature = "embassy-time", not(any(feature = "std"))))]
    #[expect(
      clippy::cast_possible_truncation,
      reason = "expected type of the method's signature"
    )]
    return Ok(Self {
      _inner: self
        ._inner
        .checked_add(embassy_time::Duration::from_micros(_duration.as_micros() as u64))
        .ok_or(CalendarError::ArithmeticOverflow)?,
    });
    #[cfg(not(any(feature = "std", feature = "embassy-time")))]
    return Err(crate::Error::CalendarError(CalendarError::InstantNeedsBackend));
  }

  /// Returns the difference if the resulting value is within bounds.
  #[inline]
  pub fn sub(&self, _duration: Duration) -> crate::Result<Self> {
    #[cfg(feature = "std")]
    return Ok(Self {
      _inner: self._inner.checked_sub(_duration).ok_or(CalendarError::ArithmeticOverflow)?,
    });
    #[cfg(all(feature = "embassy-time", not(any(feature = "std"))))]
    return Ok(Self {
      _inner: self
        ._inner
        .checked_sub(embassy_time::Duration::from_secs(_duration.as_secs()))
        .ok_or(CalendarError::ArithmeticOverflow)?,
    });
    #[cfg(not(any(feature = "std", feature = "embassy-time")))]
    return Err(crate::Error::CalendarError(CalendarError::InstantNeedsBackend));
  }

  /// Returns the amount of time elapsed from another instant to this one,
  /// or an error if that instant is later than this one.
  #[inline]
  pub fn duration_since(&self, _earlier: Self) -> crate::Result<Duration> {
    #[cfg(feature = "std")]
    return Ok(
      self
        ._inner
        .duration_since(_earlier._inner)
        .map_err(|_err| CalendarError::InvalidHardwareTime)?,
    );
    #[cfg(all(feature = "embassy-time", not(any(feature = "std"))))]
    return Ok(Duration::from_micros(
      self
        ._inner
        .checked_duration_since(_earlier._inner)
        .ok_or(CalendarError::ArithmeticOverflow)?
        .as_micros(),
    ));
    #[cfg(not(any(feature = "std", feature = "embassy-time")))]
    return Err(crate::Error::CalendarError(CalendarError::InstantNeedsBackend));
  }

  /// Returns the amount of time elapsed since this instant was created.
  #[inline]
  pub fn elapsed(&self) -> crate::Result<Duration> {
    Self::now().duration_since(*self)
  }

  /// Non-constructor method that returns the number of non-leap seconds since the UNIX epoch based
  /// on the current instance.
  ///
  #[doc = doc_epoch!()]
  #[inline]
  pub fn timestamp(&self, secs: u64) -> crate::Result<Duration> {
    self.duration_since(Self::epoch(secs)?)
  }
}
