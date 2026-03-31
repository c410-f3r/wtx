use crate::{
  asn1::{Any, Len, Oid, SEQUENCE_TAG, asn1_writer, decode_asn1_tlv},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::X509Error,
};

/// The algorithm identifier is used to identify a cryptographic algorithm.
#[derive(Debug, PartialEq)]
pub struct AlgorithmIdentifier<'bytes> {
  /// The OID that uniquely identifies the algorithm.
  pub algorithm: Oid,
  /// Optional DER-encoded algorithm parameters (may be NULL or absent).
  pub parameters: Option<Any<&'bytes [u8]>>,
}

impl<'de> Decode<'de, GenericCodec<Option<u8>, ()>> for AlgorithmIdentifier<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Option<u8>>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidAlgorithmIdentifier.into());
    };
    dw.bytes = value;
    let algorithm = Oid::decode(dw)?;
    let parameters = if dw.bytes.is_empty() { None } else { Some(Any::decode(dw)?) };
    dw.bytes = rest;
    Ok(Self { algorithm, parameters })
  }
}

impl<'bytes> Encode<GenericCodec<(), ()>> for AlgorithmIdentifier<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, ()>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_ONE, SEQUENCE_TAG, |local_ew| {
      self.algorithm.encode(local_ew)?;
      if let Some(params) = &self.parameters {
        params.encode(local_ew)?;
      }
      Ok(())
    })
  }
}
