use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, BitString, Len, SEQUENCE_TAG, asn1_writer,
    decode_asn1_tlv,
  },
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::{AlgorithmIdentifier, TbsCertificate, X509Error},
};

/// A complete X.509 certificate comprising the signed data, algorithm, and signature.
#[derive(Debug, PartialEq)]
pub struct Certificate<'bytes> {
  /// See [`AlgorithmIdentifier`].
  pub signature_algorithm: AlgorithmIdentifier<'bytes>,
  /// The digital signature computed over the DER encoding of tbs_certificate.
  pub signature_value: BitString<&'bytes [u8]>,
  /// See [`TbsCertificate`].
  pub tbs_certificate: TbsCertificate<'bytes>,
}

impl<'bytes> Certificate<'bytes> {
  /// From DER bytes
  pub fn from_der(bytes: &'bytes [u8]) -> crate::Result<Self> {
    Self::decode(&mut GenericDecodeWrapper::new(bytes, Asn1DecodeWrapper::default()))
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for Certificate<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidCertificate.into());
    };
    dw.bytes = value;
    let tbs_certificate = TbsCertificate::decode(dw)?;
    let signature_algorithm = AlgorithmIdentifier::decode(dw)?;
    let signature_value = BitString::decode(dw)?;
    dw.bytes = rest;
    Ok(Self { signature_algorithm, signature_value, tbs_certificate })
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for Certificate<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_THREE_BYTES, SEQUENCE_TAG, |local_ew| {
      self.tbs_certificate.encode(local_ew)?;
      self.signature_algorithm.encode(local_ew)?;
      self.signature_value.encode(local_ew)?;
      Ok(())
    })
  }
}
