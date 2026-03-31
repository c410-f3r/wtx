use crate::{
  asn1::{Len, SEQUENCE_TAG, Time, asn1_writer, decode_asn1_tlv},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::X509Error,
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

impl<'de> Decode<'de, GenericCodec<Option<u8>, ()>> for Validity {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Option<u8>>) -> crate::Result<Self> {
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

impl Encode<GenericCodec<(), ()>> for Validity {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, ()>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_ONE, SEQUENCE_TAG, |local_ew| {
      self.not_before.encode(local_ew)?;
      self.not_after.encode(local_ew)?;
      Ok(())
    })
  }
}
