use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::RdnSequence,
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
    Ok(Self { rdn_sequence: RdnSequence::decode(dw)? })
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for Name<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    self.rdn_sequence.encode(ew)
  }
}
