use crate::{
  database::{
    DatabaseError, Typed,
    client::mysql::{DecodeWrapper, EncodeWrapper, Mysql, Ty, ty_params::TyParams},
  },
  misc::{Decode, Encode, Usize},
  time::{ClockTime, Date, DateTime},
};
use core::any::type_name;

impl<E> Decode<'_, Mysql<E>> for Date
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(_: &mut (), dw: &mut DecodeWrapper<'_>) -> Result<Self, E> {
    date_decode(dw).map(|el| el.1).map_err(E::from)
  }
}
impl<E> Encode<Mysql<E>> for Date
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_>) -> Result<(), E> {
    date_encode(self, ew, 4).map_err(E::from)
  }
}
impl<E> Typed<Mysql<E>> for Date
where
  E: From<crate::Error>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<TyParams> {
    <Self as Typed<Mysql<E>>>::static_ty()
  }

  #[inline]
  fn static_ty() -> Option<TyParams> {
    Some(TyParams::binary(Ty::Date))
  }
}

impl<E> Decode<'_, Mysql<E>> for DateTime
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(_: &mut (), dw: &mut DecodeWrapper<'_>) -> Result<Self, E> {
    let (len, date, bytes) = date_decode(dw).map_err(E::from)?;
    Ok(if len > 4 {
      Self::new(date, time_decode(bytes)?)
    } else {
      Self::new(date, ClockTime::default())
    })
  }
}
impl<E> Encode<Mysql<E>> for DateTime
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_>) -> Result<(), E> {
    let len = date_len(&self.time());
    date_encode(&self.date(), ew, len)?;
    if len > 4 {
      time_encode(&self.time(), len > 7, ew)?;
    }
    Ok(())
  }
}
impl<E> Typed<Mysql<E>> for DateTime
where
  E: From<crate::Error>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<TyParams> {
    <Self as Typed<Mysql<E>>>::static_ty()
  }

  #[inline]
  fn static_ty() -> Option<TyParams> {
    Some(TyParams::binary(Ty::Datetime))
  }
}

fn date_decode<'de>(dw: &mut DecodeWrapper<'de>) -> crate::Result<(u8, Date, &'de [u8])> {
  let [len, year_a, year_b, month, day, rest @ ..] = dw.bytes() else {
    return Err(
      DatabaseError::UnexpectedBufferSize {
        expected: 5,
        received: Usize::from(dw.bytes().len()).into_saturating_u32(),
      }
      .into(),
    );
  };
  let year = i16::from_le_bytes([*year_a, *year_b]);
  let date = Date::from_ymd(year.try_into()?, (*month).try_into()?, (*day).try_into()?)?;
  Ok((*len, date, rest))
}

fn date_encode(date: &Date, ew: &mut EncodeWrapper<'_>, len: u8) -> crate::Result<()> {
  let year = u16::try_from(date.year().num())
    .map_err(|_err| DatabaseError::UnexpectedValueFromBytes { expected: type_name::<Date>() })?;
  let [year_a, year_b] = year.to_le_bytes();
  ew.buffer().extend_from_copyable_slice(&[
    len,
    year_a,
    year_b,
    date.month().num(),
    date.day().num(),
  ])?;
  Ok(())
}

fn date_len(time: &ClockTime) -> u8 {
  match (time.hour().num(), time.minute().num(), time.second().num(), time.nanosecond().num()) {
    (0, 0, 0, 0) => 4,
    (_, _, _, 0) => 7,
    (_, _, _, _) => 11,
  }
}

fn time_decode(bytes: &[u8]) -> crate::Result<ClockTime> {
  let (hours, minutes, seconds, micros) = if let [hours, minutes, seconds, a, b, c, d] = bytes {
    (*hours, *minutes, *seconds, u32::from_le_bytes([*a, *b, *c, *d]))
  } else if let [hours, minutes, seconds] = bytes {
    (*hours, *minutes, *seconds, 0)
  } else {
    return Err(
      DatabaseError::UnexpectedBufferSize {
        expected: 3,
        received: Usize::from(bytes.len()).into_saturating_u32(),
      }
      .into(),
    );
  };
  Ok(ClockTime::from_hms_us(
    hours.try_into()?,
    minutes.try_into()?,
    seconds.try_into()?,
    micros.try_into()?,
  ))
}

fn time_encode(
  time: &ClockTime,
  include_micros: bool,
  ew: &mut EncodeWrapper<'_>,
) -> crate::Result<()> {
  let hour = time.hour().num();
  let minute = time.minute().num();
  let second = time.second().num();
  if include_micros {
    let [a, b, c, d] = (time.nanosecond().num() / 1000).to_le_bytes();
    ew.buffer().extend_from_copyable_slice(&[hour, minute, second, a, b, c, d])?;
  } else {
    ew.buffer().extend_from_copyable_slice(&[hour, minute, second])?;
  }
  Ok(())
}

test!(
  date,
  Date,
  Date::from_ymd(2024.try_into().unwrap(), crate::time::Month::January, crate::time::Day::N6)
    .unwrap()
);
test!(datetime, DateTime, DateTime::EPOCH);
