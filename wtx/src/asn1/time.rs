use crate::{
  asn1::{Asn1Error, GENERALIZED_TIME_TAG, Len, UTC_TIME_TAG, decode_asn1_tlv},
  calendar::{Date, DateTime, Day, Hour, Minute, Month, Second, Utc, Year},
  codec::{Decode, Encode, FromRadix10, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
};

/// X509 time, which has two different representations.
#[derive(Debug, PartialEq)]
pub struct Time {
  date_time: DateTime<Utc>,
  is_generalized: bool,
}

impl Time {
  /// See [`DateTime`].
  #[inline]
  pub fn date_time(&self) -> DateTime<Utc> {
    self.date_time
  }
}

impl<'de> Decode<'de, GenericCodec<Option<u8>, ()>> for Time {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Option<u8>>) -> crate::Result<Self> {
    let (tag, _, value, rest) = decode_asn1_tlv(dw.bytes)?;
    let (date_time, is_generalized) = match (tag, value) {
      (UTC_TIME_TAG, [a, b, c, d, e, f, g, h, i, j, k, l, b'Z']) => {
        let mut year = i16::from_radix_10(&[*a, *b])?;
        year = if year >= 50 { 1900i16.wrapping_add(year) } else { 2000i16.wrapping_add(year) };
        (parse_datetime(year, [c, d, e, f, g, h, i, j, k, l])?, false)
      }
      (GENERALIZED_TIME_TAG, [a, b, c, d, e, f, g, h, i, j, k, l, m, n, b'Z']) => {
        let year = i16::from_radix_10(&[*a, *b, *c, *d])?;
        (parse_datetime(year, [e, f, g, h, i, j, k, l, m, n])?, true)
      }
      _ => return Err(Asn1Error::InvalidTime.into()),
    };
    dw.bytes = rest;
    Ok(Self { date_time, is_generalized })
  }
}

impl Encode<GenericCodec<(), ()>> for Time {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, ()>) -> crate::Result<()> {
    let year = self.date_time.date().year().num_str();
    let (actual_year, len, tag) = if self.is_generalized {
      (year.as_str(), 15, GENERALIZED_TIME_TAG)
    } else {
      (year.get(2..).unwrap_or_default(), 13, UTC_TIME_TAG)
    };
    let _ = ew.buffer.extend_from_copyable_slices([
      &[tag][..],
      &*Len::from_usize(0, len)?,
      actual_year.as_bytes(),
      self.date_time.date().month().num_str().as_bytes(),
      self.date_time.date().day().num_str().as_bytes(),
      self.date_time.time().hour().num_str().as_bytes(),
      self.date_time.time().minute().num_str().as_bytes(),
      self.date_time.time().second().num_str().as_bytes(),
      b"Z",
    ])?;
    Ok(())
  }
}

#[inline]
fn parse_datetime(year: i16, bytes: [&u8; 10]) -> crate::Result<DateTime<Utc>> {
  let [month0, month1, day0, day1, hour0, hour1, min0, min1, sec0, sec1] = bytes;
  let date = Date::from_ymd(
    Year::from_num(year)?,
    Month::from_num(u8::from_radix_10(&[*month0, *month1])?)?,
    Day::from_num(u8::from_radix_10(&[*day0, *day1])?)?,
  )?;
  let time = crate::calendar::Time::from_hms(
    Hour::from_num(u8::from_radix_10(&[*hour0, *hour1])?)?,
    Minute::from_num(u8::from_radix_10(&[*min0, *min1])?)?,
    Second::from_num(u8::from_radix_10(&[*sec0, *sec1])?)?,
  );
  Ok(DateTime::new(date, time, Utc))
}
