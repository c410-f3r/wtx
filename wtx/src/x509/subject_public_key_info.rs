use crate::{
  asn1::{
    Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, BitString, Len, OID_PKCS1_RSASSAPSS, Oid,
    SEQUENCE_TAG, asn1_writer, decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  misc::Lease,
  x509::{AlgorithmIdentifier, RsassaPssParams, X509Error},
};

/// Used to carry the public key and identify the algorithm with which the key is used
/// (e.g., RSA, DSA, or Diffie-Hellman).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SubjectPublicKeyInfo<B> {
  /// See [`AlgorithmIdentifier`].
  pub algorithm: AlgorithmIdentifier<B>,
  /// Content of the public key.
  pub subject_public_key: BitString<B>,
}

impl<B> SubjectPublicKeyInfo<B> {
  /// Shortcut
  #[inline]
  pub const fn new(algorithm: AlgorithmIdentifier<B>, subject_public_key: BitString<B>) -> Self {
    Self { algorithm, subject_public_key }
  }

  /// Additional algorithm metadata
  #[inline]
  pub fn params_oid(&self) -> Option<Oid>
  where
    B: Lease<[u8]>,
  {
    let bytes = self.algorithm.parameters.as_ref()?.bytes();
    let mut dw = DecodeWrapper::new(bytes.lease(), Asn1DecodeWrapperAux::default());
    if self.algorithm.algorithm == OID_PKCS1_RSASSAPSS {
      Some(RsassaPssParams::<&[u8]>::decode(&mut dw).ok()?.hash_algorithm?.algorithm)
    } else {
      Oid::decode(&mut dw).ok()
    }
  }
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for SubjectPublicKeyInfo<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
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

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for SubjectPublicKeyInfo<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_THREE_BYTES, SEQUENCE_TAG, |local_ew| {
      self.algorithm.encode(local_ew)?;
      self.subject_public_key.encode(local_ew)?;
      Ok(())
    })
  }
}
