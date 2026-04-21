use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, Asn1Error, Len, UTC_TIME_TAG, decode_asn1_tlv,
    parse_datetime,
  },
  calendar::{DateTime, Utc},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, FromRadix10, GenericCodec},
};

/// X509 time, which has two different representations.
#[derive(Debug, PartialEq)]
pub struct UtcTime(
  /// See [`DateTime`].
  pub DateTime<Utc>,
);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for UtcTime {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (UTC_TIME_TAG, _, [a, b, c, d, e, f, g, h, i, j, k, l, b'Z'], rest) =
      decode_asn1_tlv(dw.bytes)?
    else {
      return Err(Asn1Error::InvalidUtcTime.into());
    };
    let mut year = i16::from_radix_10(&[*a, *b])?;
    year = if year >= 50 { 1900i16.wrapping_add(year) } else { 2000i16.wrapping_add(year) };
    let value = parse_datetime(year, [c, d, e, f, g, h, i, j, k, l])?;
    dw.bytes = rest;
    Ok(Self(value))
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for UtcTime {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    let _ = ew.buffer.extend_from_copyable_slices([
      &[UTC_TIME_TAG][..],
      &*Len::from_usize(0, 13)?,
      self.0.date().year().num_str().as_bytes().get(2..).unwrap_or_default(),
      self.0.date().month().num_str().as_bytes(),
      self.0.date().day().num_str().as_bytes(),
      self.0.time().hour().num_str().as_bytes(),
      self.0.time().minute().num_str().as_bytes(),
      self.0.time().second().num_str().as_bytes(),
      b"Z",
    ])?;
    Ok(())
  }
}
