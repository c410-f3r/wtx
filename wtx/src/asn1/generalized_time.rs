use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, Asn1Error, GENERALIZED_TIME_TAG, Len, decode_asn1_tlv,
    parse_datetime,
  },
  calendar::{DateTime, Utc},
  codec::{Decode, Encode, FromRadix10, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
};

/// X509 time, which has two different representations.
#[derive(Debug, PartialEq)]
pub struct GeneralizedTime(
  /// See [`DateTime`].
  pub DateTime<Utc>,
);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for GeneralizedTime {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (GENERALIZED_TIME_TAG, _, [a, b, c, d, e, f, g, h, i, j, k, l, m, n, b'Z'], rest) =
      decode_asn1_tlv(dw.bytes)?
    else {
      return Err(Asn1Error::InvalidGeneralizedTime.into());
    };
    let year = i16::from_radix_10(&[*a, *b, *c, *d])?;
    let value = parse_datetime(year, [e, f, g, h, i, j, k, l, m, n])?;
    dw.bytes = rest;
    Ok(Self(value))
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for GeneralizedTime {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    let _ = ew.buffer.extend_from_copyable_slices([
      &[GENERALIZED_TIME_TAG][..],
      &*Len::from_usize(0, 15)?,
      self.0.date().year().num_str().as_bytes(),
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
