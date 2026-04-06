use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, BitString, Len, SEQUENCE_TAG, asn1_writer,
    decode_asn1_tlv,
  },
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::{AlgorithmIdentifier, TbsCertList, X509Error},
};

/// A digitally signed, time-stamped list published by a root CA containing revoked digital
/// certificates.
#[derive(Debug, PartialEq)]
pub struct Crl<'bytes> {
  /// See [`TbsCertList`].
  pub tbs_cert_list: TbsCertList<'bytes>,
  /// See [`AlgorithmIdentifier`].
  pub signature_algorithm: AlgorithmIdentifier<'bytes>,
  /// Digital signature computed upon the ASN.1 DER encoded [`TbsCertList`].
  pub signature_value: BitString<&'bytes [u8]>,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for Crl<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidCertificateList.into());
    };
    dw.bytes = value;
    let tbs_cert_list = TbsCertList::decode(dw)?;
    let signature_algorithm = AlgorithmIdentifier::decode(dw)?;
    let signature_value = BitString::decode(dw)?;
    dw.bytes = rest;
    Ok(Self { signature_algorithm, signature_value, tbs_cert_list })
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for Crl<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_THREE_BYTES, SEQUENCE_TAG, |local_ew| {
      self.tbs_cert_list.encode(local_ew)?;
      self.signature_algorithm.encode(local_ew)?;
      self.signature_value.encode(local_ew)?;
      Ok(())
    })
  }
}
