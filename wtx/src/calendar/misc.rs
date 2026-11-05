// FIXME(STABLE): Constant traits

#[cfg(feature = "serde")]
use crate::calendar::{Datetime, TimeZone};
use crate::{
  codec::nstr::{NStr32, NumStrFmt, nstr32_fmt},
  misc::AsciiGraphic,
};

/// Serializes a datetime as an ISO 8601 string without timezone information.
#[cfg(feature = "serde")]
#[inline]
pub fn serde_serialize_datetime_iso8601_without_tz<S, TZ>(
  datetime: &Datetime<TZ>,
  serializer: S,
) -> Result<S::Ok, S::Error>
where
  S: serde::Serializer,
  TZ: TimeZone,
{
  serializer.serialize_str(&datetime.iso8601(false))
}

/// Serializes an optional datetime as an ISO 8601 string without timezone information.
#[cfg(feature = "serde")]
#[inline]
pub fn serde_serialize_datetime_iso8601_without_tz_opt<S, TZ>(
  datetime: &Option<Datetime<TZ>>,
  serializer: S,
) -> Result<S::Ok, S::Error>
where
  S: serde::Serializer,
  TZ: TimeZone,
{
  match datetime {
    Some(elem) => serializer.serialize_str(&elem.iso8601(false)),
    None => serializer.serialize_none(),
  }
}

pub(crate) fn nanosecond_string(nanosecond: u32) -> NStr32 {
  nstr32_fmt(nanosecond, NumStrFmt::new().pad(AsciiGraphic::ZERO, 9))
}
