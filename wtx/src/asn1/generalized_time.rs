use crate::{
  asn1::{
    Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Asn1Error, GENERALIZED_TIME_TAG, Len,
    decode_asn1_tlv, parse_datetime,
  },
  calendar::{DateTime, Utc},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, FromRadix10 as _, GenericCodec},
};

/// X509 time, which has two different representations.
#[derive(Debug, PartialEq)]
pub struct GeneralizedTime(
  /// See [`DateTime`].
  pub DateTime<Utc>,
);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for GeneralizedTime {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let (
      GENERALIZED_TIME_TAG,
      _,
      [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b'Z'],
      rest,
    ) = decode_asn1_tlv(dw.bytes)?
    else {
      return Err(Asn1Error::InvalidGeneralizedTime.into());
    };
    let year = i16::from_radix_10(&[*b0, *b1, *b2, *b3])?;
    let value = parse_datetime(year, [b4, b5, b6, b7, b8, b9, b10, b11, b12, b13])?;
    dw.bytes = rest;
    Ok(Self(value))
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for GeneralizedTime {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
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
