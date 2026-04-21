use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, BitString, Len, SEQUENCE_TAG, asn1_writer,
    decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  x509::{AlgorithmIdentifier, X509Error},
};

/// Used to carry the public key and identify the algorithm with which the key is used
/// (e.g., RSA, DSA, or Diffie-Hellman).
#[derive(Clone, Debug, PartialEq)]
pub struct SubjectPublicKeyInfo<'bytes> {
  /// See [`AlgorithmIdentifier`].
  pub algorithm: AlgorithmIdentifier<'bytes>,
  /// Content of the public key.
  pub subject_public_key: BitString<&'bytes [u8]>,
}

impl<'bytes> SubjectPublicKeyInfo<'bytes> {
  /// Shortcut
  pub const fn new(
    algorithm: AlgorithmIdentifier<'bytes>,
    subject_public_key: BitString<&'bytes [u8]>,
  ) -> Self {
    Self { algorithm, subject_public_key }
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for SubjectPublicKeyInfo<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
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

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for SubjectPublicKeyInfo<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_THREE_BYTES, SEQUENCE_TAG, |local_ew| {
      self.algorithm.encode(local_ew)?;
      self.subject_public_key.encode(local_ew)?;
      Ok(())
    })
  }
}
