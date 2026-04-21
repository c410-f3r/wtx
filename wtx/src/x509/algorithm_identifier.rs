use crate::{
  asn1::{
    Any, Asn1DecodeWrapper, Asn1EncodeWrapper, Len, Oid, SEQUENCE_TAG, asn1_writer, decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  x509::X509Error,
};

/// The algorithm identifier is used to identify a cryptographic algorithm.
#[derive(Clone, Debug, PartialEq)]
pub struct AlgorithmIdentifier<'bytes> {
  /// The OID that uniquely identifies the algorithm.
  pub algorithm: Oid,
  /// Optional DER-encoded algorithm parameters (may be NULL or absent).
  pub parameters: Option<Any<&'bytes [u8]>>,
}

impl<'bytes> AlgorithmIdentifier<'bytes> {
  /// Shortcut
  pub const fn from_algorithm(algorithm: Oid) -> Self {
    Self { algorithm, parameters: None }
  }

  /// Shortcut
  pub const fn new(algorithm: Oid, parameters: Option<Any<&'bytes [u8]>>) -> Self {
    Self { algorithm, parameters }
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for AlgorithmIdentifier<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
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

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for AlgorithmIdentifier<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_ONE_BYTE, SEQUENCE_TAG, |local_ew| {
      self.algorithm.encode(local_ew)?;
      if let Some(params) = &self.parameters {
        params.encode(local_ew)?;
      }
      Ok(())
    })
  }
}
