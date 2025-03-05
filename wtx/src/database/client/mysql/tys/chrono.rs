use crate::{
  database::{
    DatabaseError, Typed,
    client::mysql::{DecodeWrapper, EncodeWrapper, Mysql, Ty, ty_params::TyParams},
  },
  misc::{Decode, Encode, Usize},
};
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc};
use core::any::type_name;

impl<E> Decode<'_, Mysql<E>> for DateTime<Utc>
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(aux: &mut (), dv: &mut DecodeWrapper<'_>) -> Result<Self, E> {
    let naive = <NaiveDateTime as Decode<Mysql<E>>>::decode(aux, dv)?;
    Ok(Utc.from_utc_datetime(&naive))
  }
}
impl<E> Encode<Mysql<E>> for DateTime<Utc>
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, aux: &mut (), ew: &mut EncodeWrapper<'_>) -> Result<(), E> {
    Encode::<Mysql<E>>::encode(&self.naive_utc(), aux, ew)
  }
}
impl<E> Typed<Mysql<E>> for DateTime<Utc>
where
  E: From<crate::Error>,
{
  const TY: Option<TyParams> = Some(TyParams::binary(Ty::Timestamp));
}

impl<E> Decode<'_, Mysql<E>> for NaiveDate
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(_: &mut (), dv: &mut DecodeWrapper<'_>) -> Result<Self, E> {
    date_decode(dv).map(|el| el.1).map_err(E::from)
  }
}
impl<E> Encode<Mysql<E>> for NaiveDate
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_>) -> Result<(), E> {
    date_encode(self, ew, 4).map_err(E::from)
  }
}
impl<E> Typed<Mysql<E>> for NaiveDate
where
  E: From<crate::Error>,
{
  const TY: Option<TyParams> = Some(TyParams::binary(Ty::Date));
}

impl<E> Decode<'_, Mysql<E>> for NaiveDateTime
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(_: &mut (), dv: &mut DecodeWrapper<'_>) -> Result<Self, E> {
    let (len, date, bytes) = date_decode(dv).map_err(E::from)?;
    Ok(if len > 4 {
      date.and_time(time_decode(bytes)?)
    } else {
      date.and_time(NaiveTime::default())
    })
  }
}
impl<E> Encode<Mysql<E>> for NaiveDateTime
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_>) -> Result<(), E> {
    let len = date_len(self);
    date_encode(&self.date(), ew, len)?;
    if len > 4 {
      time_encode(&self.time(), len > 7, ew)?;
    }
    Ok(())
  }
}
impl<E> Typed<Mysql<E>> for NaiveDateTime
where
  E: From<crate::Error>,
{
  const TY: Option<TyParams> = Some(TyParams::binary(Ty::Datetime));
}

#[inline]
fn date_decode<'de>(dv: &mut DecodeWrapper<'de>) -> crate::Result<(u8, NaiveDate, &'de [u8])> {
  let [len, year_a, year_b, month, day, rest @ ..] = dv.bytes() else {
    return Err(
      DatabaseError::UnexpectedBufferSize {
        expected: 5,
        received: Usize::from(dv.bytes().len()).into_u32().unwrap_or(u32::MAX),
      }
      .into(),
    );
  };
  let year = i16::from_le_bytes([*year_a, *year_b]);
  let date =
    NaiveDate::from_ymd_opt(year.into(), (*month).into(), (*day).into()).ok_or_else(|| {
      DatabaseError::UnexpectedValueFromBytes { expected: type_name::<NaiveDate>() }
    })?;
  Ok((*len, date, rest))
}

#[inline]
fn date_encode(date: &NaiveDate, ew: &mut EncodeWrapper<'_>, len: u8) -> crate::Result<()> {
  let year = u16::try_from(date.year()).map_err(|_err| {
    DatabaseError::UnexpectedValueFromBytes { expected: type_name::<NaiveDate>() }
  })?;
  let [year_a, year_b] = year.to_le_bytes();
  ew.sw().extend_from_copyable_slice(&[
    len,
    year_a,
    year_b,
    date.month().try_into().unwrap_or_default(),
    date.day().try_into().unwrap_or_default(),
  ])?;
  Ok(())
}

#[inline]
fn date_len(time: &NaiveDateTime) -> u8 {
  match (time.hour(), time.minute(), time.second(), time.and_utc().timestamp_subsec_nanos()) {
    (0, 0, 0, 0) => 4,
    (_, _, _, 0) => 7,
    (_, _, _, _) => 11,
  }
}

#[inline]
fn time_decode(bytes: &[u8]) -> crate::Result<NaiveTime> {
  let (hours, minutes, seconds, micros) = if let [hours, minutes, seconds, a, b, c, d] = bytes {
    (*hours, *minutes, *seconds, u32::from_le_bytes([*a, *b, *c, *d]))
  } else if let [hours, minutes, seconds] = bytes {
    (*hours, *minutes, *seconds, 0)
  } else {
    return Err(
      DatabaseError::UnexpectedBufferSize {
        expected: 3,
        received: Usize::from(bytes.len()).into_u32().unwrap_or(u32::MAX),
      }
      .into(),
    );
  };
  NaiveTime::from_hms_micro_opt(hours.into(), minutes.into(), seconds.into(), micros).ok_or_else(
    || DatabaseError::UnexpectedValueFromBytes { expected: type_name::<NaiveTime>() }.into(),
  )
}

fn time_encode(
  time: &NaiveTime,
  include_micros: bool,
  ew: &mut EncodeWrapper<'_>,
) -> crate::Result<()> {
  let hour = time.hour().try_into().unwrap_or_default();
  let minute = time.minute().try_into().unwrap_or_default();
  let second = time.second().try_into().unwrap_or_default();
  if include_micros {
    let [a, b, c, d] = (time.nanosecond() / 1000).to_le_bytes();
    ew.sw().extend_from_copyable_slice(&[hour, minute, second, a, b, c, d])?;
  } else {
    ew.sw().extend_from_copyable_slice(&[hour, minute, second])?;
  }
  Ok(())
}

test!(datetime_utc, DateTime<Utc>, "2025-02-27T16:26:06.438497Z".parse().unwrap());
