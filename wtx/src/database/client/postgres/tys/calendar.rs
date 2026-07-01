use crate::{
  calendar::{Date, DateTime, Day, Month, Nanosecond, SECONDS_PER_DAY, Time, Utc, Year},
  codec::{Decode, Encode},
  collections::ShortStrU8,
  database::{
    DatabaseError, Typed,
    client::postgres::{Postgres, PostgresDecodeWrapper, PostgresEncodeWrapper, PostgresError, Ty},
  },
  misc::TryArithmetic as _,
};

const PG_EPOCH: DateTime<Utc> = DateTime::new(
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
  Utc,
);
const PG_MIN: DateTime<Utc> = DateTime::new(
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
  Utc,
);

impl<E> Decode<'_, Postgres<E>> for DateTime<Utc>
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut PostgresDecodeWrapper<'_, '_>) -> Result<Self, E> {
    let micros: i64 = Decode::<Postgres<E>>::decode(dw)?;
    let (epoch_secs, _) = PG_EPOCH.timestamp_secs_and_ns();
    let this_secs = micros.div_euclid(1_000_000);
    let this_nanoseconds = u32::try_from(micros.rem_euclid(1_000_000)).map_err(From::from)?;
    let ts_diff = epoch_secs.wrapping_add(this_secs);
    Ok(
      DateTime::from_timestamp_secs_and_ns(
        ts_diff,
        Nanosecond::from_num(this_nanoseconds.wrapping_mul(1_000)).map_err(crate::Error::from)?,
      )
      .map_err(crate::Error::from)?,
    )
  }
}
impl<E> Encode<Postgres<E>> for DateTime<Utc>
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, ew: &mut PostgresEncodeWrapper<'_>) -> Result<(), E> {
    if self < &PG_MIN || self > &DateTime::MAX {
      return Err(E::from(PostgresError::TimeStructureOverflow.into()));
    }
    let (this_secs, this_ns) = self.timestamp_secs_and_ns();
    if !this_ns.num().is_multiple_of(1_000) {
      return Err(E::from(PostgresError::TimeStructureWithGreaterPrecision.into()));
    }
    let this_microseconds = this_ns.num() / 1_000;
    let (epoch_secs, _) = PG_EPOCH.timestamp_secs_and_ns();
    let secs_diff = this_secs.wrapping_sub(epoch_secs).wrapping_mul(1_000_000);
    let rslt = secs_diff.wrapping_add(this_microseconds.into());
    Encode::<Postgres<E>>::encode(&rslt, ew)
  }
}
impl<E> Typed<Postgres<E>> for DateTime<Utc>
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
  fn decode(dw: &mut PostgresDecodeWrapper<'_, '_>) -> Result<Self, E> {
    let days: i32 = Decode::<Postgres<E>>::decode(dw)?;
    let days_in_secs = i64::from(SECONDS_PER_DAY).wrapping_mul(days.into());
    let timestamp = days_in_secs.wrapping_add(PG_EPOCH.timestamp_secs_and_ns().0);
    Ok(DateTime::from_timestamp_secs(timestamp).map_err(crate::Error::from)?.date())
  }
}

impl<E> Encode<Postgres<E>> for Date
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, ew: &mut PostgresEncodeWrapper<'_>) -> Result<(), E> {
    if self < &PG_MIN.date() || self > &Date::MAX {
      return Err(E::from(
        DatabaseError::UnexpectedValueFromBytes { expected: ShortStrU8::new_truncated_u8("date") }
          .into(),
      ));
    }
    let this_timestamp = DateTime::new(*self, Time::ZERO, Utc).timestamp_secs_and_ns().0;
    let diff = this_timestamp.wrapping_sub(PG_EPOCH.timestamp_secs_and_ns().0);
    let days =
      i32::try_from(diff.try_div(i64::from(SECONDS_PER_DAY))?).map_err(crate::Error::from)?;
    Encode::<Postgres<E>>::encode(&days, ew)
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
  DateTime<Utc>,
  DateTime::from_timestamp_secs_and_ns(123456789, Nanosecond::from_num(12000).unwrap()).unwrap()
);
