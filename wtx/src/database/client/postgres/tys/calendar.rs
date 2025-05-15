use crate::{
  calendar::{Date, DateTime, Day, Month, Nanosecond, SECONDS_PER_DAY, Time, Year},
  database::{
    DatabaseError, Typed,
    client::postgres::{DecodeWrapper, EncodeWrapper, Postgres, Ty},
  },
  misc::{Decode, Encode},
};

const PG_EPOCH: DateTime = DateTime::new(
  if let Ok(date) = Date::from_ymd(
    if let Ok(year) = Year::from_num(2000) {
      year
    } else {
      panic!();
    },
    Month::January,
    Day::N1,
  ) {
    date
  } else {
    panic!();
  },
  Time::ZERO,
);
const PG_MIN: DateTime = DateTime::new(
  if let Ok(date) = Date::from_ymd(
    if let Ok(year) = Year::from_num(-4713) {
      year
    } else {
      panic!();
    },
    Month::January,
    Day::N1,
  ) {
    date
  } else {
    panic!();
  },
  Time::ZERO,
);

impl<E> Decode<'_, Postgres<E>> for DateTime
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(aux: &mut (), dw: &mut DecodeWrapper<'_>) -> Result<Self, E> {
    let micros: i64 = Decode::<Postgres<E>>::decode(aux, dw)?;
    let (epoch_ts, _) = PG_EPOCH.timestamp();
    let this_ts = micros.div_euclid(1_000_000);
    let this_ns = ((micros.rem_euclid(1_000_000)) as u32).wrapping_mul(1_000);
    let ts_diff = epoch_ts.wrapping_add(this_ts);
    Ok(DateTime::from_timestamp_secs_and_ns(
      ts_diff,
      Nanosecond::from_num(this_ns).map_err(crate::Error::from)?,
    )?)
  }
}
impl<E> Encode<Postgres<E>> for DateTime
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    if self < &PG_MIN || self > &DateTime::MAX {
      return Err(E::from(
        DatabaseError::UnexpectedValueFromBytes { expected: "timestamp" }.into(),
      ));
    }
    let (this_ts, this_ns) = self.timestamp();
    if this_ns.num() % 1_000 > 0 {
      return Err(E::from(
        DatabaseError::UnexpectedValueFromBytes { expected: "timestamp" }.into(),
      ));
    }
    let this_us = this_ns.num() / 1_000;
    let (epoch_ts, _) = PG_EPOCH.timestamp();
    let ts_diff = this_ts.wrapping_sub(epoch_ts).wrapping_mul(1_000_000);
    let rslt = ts_diff.wrapping_add(this_us.into());
    Encode::<Postgres<E>>::encode(&rslt, &mut (), ew)
  }
}
impl<E> Typed<Postgres<E>> for DateTime
where
  E: From<crate::Error>,
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

impl<E> Decode<'_, Postgres<E>> for Date
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(aux: &mut (), dw: &mut DecodeWrapper<'_>) -> Result<Self, E> {
    let days: i32 = Decode::<Postgres<E>>::decode(aux, dw)?;
    let days_in_secs = i64::from(SECONDS_PER_DAY).wrapping_mul(days.into());
    let timestamp = days_in_secs.wrapping_add(PG_EPOCH.timestamp().0);
    Ok(DateTime::from_timestamp_secs(timestamp).map_err(crate::Error::from)?.date())
  }
}

impl<E> Encode<Postgres<E>> for Date
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    if self < &PG_MIN.date() || self > &Date::MAX {
      return Err(E::from(DatabaseError::UnexpectedValueFromBytes { expected: "date" }.into()));
    }
    let this_timestamp = DateTime::new(*self, Time::ZERO).timestamp().0;
    let diff = this_timestamp.wrapping_sub(PG_EPOCH.timestamp().0);
    let days = i32::try_from(diff / i64::from(SECONDS_PER_DAY)).map_err(crate::Error::from)?;
    Encode::<Postgres<E>>::encode(&days, &mut (), ew)
  }
}
impl<E> Typed<Postgres<E>> for Date
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

test!(date, Date, Date::from_ymd(4.try_into().unwrap(), Month::January, Day::N6).unwrap());
test!(
  datetime,
  DateTime,
  DateTime::from_timestamp_secs_and_ns(123456789, Nanosecond::from_num(12000).unwrap()).unwrap()
);
