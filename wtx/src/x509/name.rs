use crate::{
  asn1::{Len, SEQUENCE_TAG, Set, asn1_writer, decode_asn1_tlv},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  collection::Vector,
  x509::{RdnSequence, X509Error},
};

/// Distinguished name.
#[derive(Debug, PartialEq)]
pub struct Name<'bytes> {
  /// See [`RdnSequence`].
  pub rdn_sequence: RdnSequence<'bytes>,
}

impl<'de> Decode<'de, GenericCodec<Option<u8>, ()>> for Name<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Option<u8>>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidName.into());
    };
    dw.bytes = value;
    let mut rdn_sequence = Vector::default();
    while !dw.bytes.is_empty() {
      rdn_sequence.push(Set::decode(dw)?.0)?;
    }
    dw.bytes = rest;
    Ok(Self { rdn_sequence })
  }
}

impl<'bytes> Encode<GenericCodec<(), ()>> for Name<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, ()>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_ONE, SEQUENCE_TAG, |local_ew| {
      for rdn in self.rdn_sequence.iter() {
        Set(rdn).encode(local_ew)?;
      }
      Ok(())
    })
  }
}
