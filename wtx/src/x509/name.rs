use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, SEQUENCE_TAG, decode_asn1_tlv},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::{RdnSequence, X509Error},
};

/// Distinguished name.
#[derive(Debug, PartialEq)]
pub struct Name<'bytes> {
  /// See [`RdnSequence`].
  pub rdn_sequence: RdnSequence<'bytes>,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for Name<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidName.into());
    };
    dw.bytes = value;
    let rdn_sequence = RdnSequence::decode(dw)?;
    dw.bytes = rest;
    Ok(Self { rdn_sequence })
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for Name<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    self.rdn_sequence.encode(ew)
  }
}
