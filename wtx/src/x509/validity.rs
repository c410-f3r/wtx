use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Len, SEQUENCE_TAG, asn1_writer, decode_asn1_tlv},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::{Time, X509Error},
};

/// Time interval during which the CA warrants that it will maintain information about the status
/// of the certificate.
#[derive(Debug, PartialEq)]
pub struct Validity {
  /// The earliest date/time at which the certificate is considered valid.
  pub not_before: Time,
  /// The latest date/time at which the certificate is considered valid.
  pub not_after: Time,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for Validity {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidValidity.into());
    };
    dw.bytes = value;
    let not_before = Time::decode(dw)?;
    let not_after = Time::decode(dw)?;
    dw.bytes = rest;
    Ok(Self { not_before, not_after })
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for Validity {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_ONE_BYTE, SEQUENCE_TAG, |local_ew| {
      self.not_before.encode(local_ew)?;
      self.not_after.encode(local_ew)?;
      Ok(())
    })
  }
}
