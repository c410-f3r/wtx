use crate::calendar::CalendarError;
use core::time::Duration;

type LocalTy = cfg_select! {
  feature = "std" => std::time::SystemTime,
  feature = "embassy-time" => embassy_time::Instant,
  _ => ()
};

/// Time provider suitable for operations involving intervals.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Instant {
  _inner: LocalTy,
}

impl Instant {
  /// Doesn't necessarily represent the current time. For example, `embassy-time` returns the time
  /// since boot.
  ///
  /// You should call converting methods like [`Self::timestamp`] to portray the present reality.
  #[inline]
  pub fn new() -> Self {
    cfg_select! {
      feature = "std" => Self { _inner: std::time::SystemTime::now() },
      feature = "embassy-time" => Self { _inner: embassy_time::Instant::now() },
      _ => Self { _inner: () }
    }
  }

  /// Returns a new `DateTime` instance with the current date and time in UTC based on the
  /// UNIX epoch.
  #[inline]
  pub fn now_date_time() -> crate::Result<crate::calendar::DateTime<crate::calendar::Utc>> {
    let timestamp = Instant::now_timestamp()?;
    Ok(crate::calendar::DateTime::from_timestamp_secs_and_ns(
      timestamp.as_secs().cast_signed(),
      timestamp.subsec_nanos().try_into()?,
    )?)
  }

  /// Constructor that returns the number of non-leap seconds since the UNIX epoch.
  #[inline]
  pub fn now_timestamp() -> crate::Result<Duration> {
    cfg_select! {
      feature = "std" => Self::new().duration_since(Self { _inner: std::time::UNIX_EPOCH }),
      feature = "embassy-time" => Ok(embassy_timestamp(embassy_time::Instant::now().as_secs())),
      _ => Err(crate::Error::CalendarError(CalendarError::InstantNeedsBackend))
    }
  }

  /// Returns the addition if the resulting value is within bounds.
  #[inline]
  pub fn add(&self, _duration: Duration) -> crate::Result<Self> {
    cfg_select! {
      feature = "std" => {
        let _inner = self._inner.checked_add(_duration).ok_or(CalendarError::ArithmeticOverflow)?;
        Ok(Self { _inner })
      },
      feature = "embassy-time" => {
        let _inner = self
          ._inner
          .checked_add(embassy_time::Duration::from_micros(_duration.as_micros() as u64))
          .ok_or(CalendarError::ArithmeticOverflow)?;
        Ok(Self { _inner })
      }
      _ => Err(crate::Error::CalendarError(CalendarError::InstantNeedsBackend))
    }
  }

  /// Returns the difference if the resulting value is within bounds.
  #[inline]
  pub fn sub(&self, _duration: Duration) -> crate::Result<Self> {
    cfg_select! {
      feature = "std" => {
        let _inner = self._inner.checked_sub(_duration).ok_or(CalendarError::ArithmeticOverflow)?;
        Ok(Self { _inner })
      },
      feature = "embassy-time" => {
        let _inner = self
          ._inner
          .checked_sub(embassy_time::Duration::from_micros(_duration.as_micros() as u64))
          .ok_or(CalendarError::ArithmeticOverflow)?;
        Ok(Self { _inner })
      },
      _ => Err(crate::Error::CalendarError(CalendarError::InstantNeedsBackend))
    }
  }

  /// Returns the amount of time elapsed from another instant to this one,
  /// or an error if that instant is later than this one.
  #[inline]
  pub fn duration_since(&self, _earlier: Self) -> crate::Result<Duration> {
    cfg_select! {
      feature = "std" => Ok(
        self
          ._inner
          .duration_since(_earlier._inner)
          .map_err(|_err| CalendarError::InvalidHardwareTime)?,
      ),
      feature = "embassy-time" => Ok(Duration::from_micros(
        self
          ._inner
          .checked_duration_since(_earlier._inner)
          .ok_or(CalendarError::ArithmeticOverflow)?
          .as_micros(),
      )),
      _ => Err(crate::Error::CalendarError(CalendarError::InstantNeedsBackend))
    }
  }

  /// Returns the amount of time elapsed since this instant was created.
  #[inline]
  pub fn elapsed(&self) -> crate::Result<Duration> {
    Self::new().duration_since(*self)
  }

  /// Non-constructor method that returns the number of non-leap seconds since the UNIX epoch based
  /// on the current instance.
  #[inline]
  pub fn timestamp(&self) -> crate::Result<Duration> {
    cfg_select! {
      feature = "std" => self.duration_since(Self { _inner: std::time::UNIX_EPOCH }),
      feature = "embassy-time" => Ok(embassy_timestamp(self._inner.as_secs())),
      _ => Err(crate::Error::CalendarError(CalendarError::InstantNeedsBackend))
    }
  }
}

impl Default for Instant {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(all(feature = "embassy-time", not(feature = "std")))]
#[inline]
fn embassy_timestamp(secs: u64) -> Duration {
  Duration::from_secs(secs.saturating_add(crate::calendar::EPOCH_OFFSET.get()))
}
