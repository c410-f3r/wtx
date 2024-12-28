use crate::{
  database::{
    client::postgres::{DecodeWrapper, EncodeWrapper, Postgres, Ty},
    DatabaseError, Typed,
  },
  misc::{Decode, Encode},
};
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, TimeDelta, TimeZone, Utc};

const MIN_PG_ND: Option<NaiveDate> = NaiveDate::from_ymd_opt(-4713, 1, 1);
const MAX_CHRONO_ND: Option<NaiveDate> = NaiveDate::from_ymd_opt(262142, 1, 1);

impl<E> Decode<'_, Postgres<E>> for DateTime<Utc>
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'_>) -> Result<Self, E> {
    let naive = <NaiveDateTime as Decode<Postgres<E>>>::decode(dw)?;
    Ok(Utc.from_utc_datetime(&naive))
  }
}

impl<E, TZ> Encode<Postgres<E>> for DateTime<TZ>
where
  E: From<crate::Error>,
  TZ: TimeZone,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    Encode::<Postgres<E>>::encode(&self.naive_utc(), ew)
  }
}

impl<E, TZ> Typed<Postgres<E>> for DateTime<TZ>
where
  E: From<crate::Error>,
  TZ: TimeZone,
{
  const TY: Ty = Ty::Timestamptz;
}

impl<E> Decode<'_, Postgres<E>> for NaiveDate
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'_>) -> Result<Self, E> {
    let days: i32 = Decode::<Postgres<E>>::decode(dw)?;
    pg_epoch_nd()
      .and_then(|el| el.checked_add_signed(TimeDelta::try_days(days.into())?))
      .ok_or_else(|| {
        E::from(DatabaseError::UnexpectedValueFromBytes { expected: "timestamp" }.into())
      })
  }
}

impl<E> Encode<Postgres<E>> for NaiveDate
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    Encode::<Postgres<E>>::encode(
      &match pg_epoch_nd().and_then(|epoch| {
        if self < &MIN_PG_ND? || self > &MAX_CHRONO_ND? {
          return None;
        }
        i32::try_from(self.signed_duration_since(epoch).num_days()).ok()
      }) {
        Some(time) => time,
        None => {
          return Err(E::from(DatabaseError::UnexpectedValueFromBytes { expected: "date" }.into()))
        }
      },
      ew,
    )
  }
}

impl<E> Typed<Postgres<E>> for NaiveDate
where
  E: From<crate::Error>,
{
  const TY: Ty = Ty::Date;
}

impl<E> Decode<'_, Postgres<E>> for NaiveDateTime
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper) -> Result<Self, E> {
    let timestamp = Decode::<Postgres<E>>::decode(dw)?;
    pg_epoch_ndt()
      .and_then(|el| el.checked_add_signed(Duration::microseconds(timestamp)))
      .ok_or_else(|| {
        E::from(DatabaseError::UnexpectedValueFromBytes { expected: "timestamp" }.into())
      })
  }
}

impl<E> Encode<Postgres<E>> for NaiveDateTime
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    Encode::<Postgres<E>>::encode(
      &match pg_epoch_ndt().and_then(|epoch| {
        if self < &MIN_PG_ND?.and_hms_opt(0, 0, 0)?
          || self > &MAX_CHRONO_ND?.and_hms_opt(0, 0, 0)?
        {
          return None;
        }
        self.signed_duration_since(epoch).num_microseconds()
      }) {
        Some(time) => time,
        None => {
          return Err(E::from(
            DatabaseError::UnexpectedValueFromBytes { expected: "timestamp" }.into(),
          ))
        }
      },
      ew,
    )
  }
}

impl<E> Typed<Postgres<E>> for NaiveDateTime
where
  E: From<crate::Error>,
{
  const TY: Ty = Ty::Timestamp;
}

fn pg_epoch_nd() -> Option<NaiveDate> {
  NaiveDate::from_ymd_opt(2000, 1, 1)
}

fn pg_epoch_ndt() -> Option<NaiveDateTime> {
  pg_epoch_nd()?.and_hms_opt(0, 0, 0)
}

test!(datetime_utc, DateTime<Utc>, Utc.from_utc_datetime(&pg_epoch_ndt().unwrap()));
