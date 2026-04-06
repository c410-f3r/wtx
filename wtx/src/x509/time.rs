use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, GENERALIZED_TIME_TAG, GeneralizedTime, UTC_TIME_TAG,
    UtcTime,
  },
  calendar::{DateTime, Utc},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::X509Error,
};

/// X509 time, which has two different representations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Time {
  date_time: DateTime<Utc>,
  tag: u8,
}

impl Time {
  /// See [`DateTime`].
  #[inline]
  pub fn date_time(&self) -> DateTime<Utc> {
    self.date_time
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for Time {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (date_time, tag) = if let Ok(elem) = GeneralizedTime::decode(dw) {
      (elem.0, GENERALIZED_TIME_TAG)
    } else if let Ok(elem) = UtcTime::decode(dw) {
      (elem.0, UTC_TIME_TAG)
    } else {
      return Err(X509Error::InvalidTime.into());
    };
    Ok(Self { date_time, tag })
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for Time {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    if self.tag == GENERALIZED_TIME_TAG {
      GeneralizedTime(self.date_time).encode(ew)?;
    } else {
      UtcTime(self.date_time).encode(ew)?;
    }
    Ok(())
  }
}
