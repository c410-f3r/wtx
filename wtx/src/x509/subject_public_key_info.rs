use crate::{
  asn1::{BitString, Len, SEQUENCE_TAG, asn1_writer, decode_asn1_tlv},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::{AlgorithmIdentifier, X509Error},
};

/// Used to carry the public key and identify the algorithm with which the key is used
/// (e.g., RSA, DSA, or Diffie-Hellman).
#[derive(Debug, PartialEq)]
pub struct SubjectPublicKeyInfo<'bytes> {
  /// See [`AlgorithmIdentifier`].
  pub algorithm: AlgorithmIdentifier<'bytes>,
  /// Content of the public key.
  pub subject_public_key: BitString<&'bytes [u8]>,
}

impl<'de> Decode<'de, GenericCodec<Option<u8>, ()>> for SubjectPublicKeyInfo<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Option<u8>>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidSubjectPublicKeyInfo.into());
    };
    dw.bytes = value;
    let algorithm = AlgorithmIdentifier::decode(dw)?;
    let subject_public_key = BitString::decode(dw)?;
    dw.bytes = rest;
    Ok(Self { algorithm, subject_public_key })
  }
}

impl<'bytes> Encode<GenericCodec<(), ()>> for SubjectPublicKeyInfo<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, ()>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_ONE, SEQUENCE_TAG, |local_ew| {
      self.algorithm.encode(local_ew)?;
      self.subject_public_key.encode(local_ew)?;
      Ok(())
    })
  }
}
