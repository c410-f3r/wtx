// FIXME(STABLE): Constant traits

#[cfg(feature = "serde")]
use crate::calendar::{Datetime, TimeZone};
use crate::{
  codec::{U32String, u32_string_pad},
  misc::{AsciiGraphic, const_ok},
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

pub(crate) fn nanosecond_string(nanosecond: u32) -> U32String {
  u32_string_pad(nanosecond, const { const_ok(AsciiGraphic::new(b'0')).unwrap() }, 9)
}
