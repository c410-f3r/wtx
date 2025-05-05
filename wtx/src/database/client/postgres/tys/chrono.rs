use crate::{
  database::{
    DatabaseError, Typed,
    client::postgres::{DecodeWrapper, EncodeWrapper, Postgres, Ty},
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
  fn decode(aux: &mut (), dw: &mut DecodeWrapper<'_>) -> Result<Self, E> {
    let naive = <NaiveDateTime as Decode<Postgres<E>>>::decode(aux, dw)?;
    Ok(Utc.from_utc_datetime(&naive))
  }
}

impl<E, TZ> Encode<Postgres<E>> for DateTime<TZ>
where
  E: From<crate::Error>,
  TZ: TimeZone,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    Encode::<Postgres<E>>::encode(&self.naive_utc(), &mut (), ew)
  }
}

impl<E, TZ> Typed<Postgres<E>> for DateTime<TZ>
where
  E: From<crate::Error>,
  TZ: TimeZone,
{
  #[inline]
  fn runtime_ty(&self) -> Option<Ty> {
    <Self as Typed<Postgres<E>>>::static_ty()
  }

  #[inline]
  fn static_ty() -> Option<Ty> {
    Some(Ty::Timestamptz)
  }
}

impl<E> Decode<'_, Postgres<E>> for NaiveDate
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(aux: &mut (), dw: &mut DecodeWrapper<'_>) -> Result<Self, E> {
    let days: i32 = Decode::<Postgres<E>>::decode(aux, dw)?;
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
  fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    Encode::<Postgres<E>>::encode(
      &match pg_epoch_nd().and_then(|epoch| {
        if self < &MIN_PG_ND? || self > &MAX_CHRONO_ND? {
          return None;
        }
        i32::try_from(self.signed_duration_since(epoch).num_days()).ok()
      }) {
        Some(time) => time,
        None => {
          return Err(E::from(DatabaseError::UnexpectedValueFromBytes { expected: "date" }.into()));
        }
      },
      &mut (),
      ew,
    )
  }
}

impl<E> Typed<Postgres<E>> for NaiveDate
where
  E: From<crate::Error>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<Ty> {
    <Self as Typed<Postgres<E>>>::static_ty()
  }

  #[inline]
  fn static_ty() -> Option<Ty> {
    Some(Ty::Date)
  }
}

impl<E> Decode<'_, Postgres<E>> for NaiveDateTime
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(aux: &mut (), dw: &mut DecodeWrapper<'_>) -> Result<Self, E> {
    let timestamp = Decode::<Postgres<E>>::decode(aux, dw)?;
    pg_epoch_ndt()
      .and_then(|epoch| epoch.checked_add_signed(Duration::microseconds(timestamp)))
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
  fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
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
          ));
        }
      },
      &mut (),
      ew,
    )
  }
}

impl<E> Typed<Postgres<E>> for NaiveDateTime
where
  E: From<crate::Error>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<Ty> {
    <Self as Typed<Postgres<E>>>::static_ty()
  }

  #[inline]
  fn static_ty() -> Option<Ty> {
    Some(Ty::Timestamp)
  }
}

fn pg_epoch_nd() -> Option<NaiveDate> {
  NaiveDate::from_ymd_opt(2000, 1, 1)
}

fn pg_epoch_ndt() -> Option<NaiveDateTime> {
  pg_epoch_nd()?.and_hms_opt(0, 0, 0)
}

test!(datetime_utc, DateTime<Utc>, Utc.from_utc_datetime(&pg_epoch_ndt().unwrap()));
