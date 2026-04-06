#[cfg(feature = "base64")]
use crate::{collection::TryExtend, misc::Pem};
#[cfg(feature = "base64")]
use core::ops::Range;

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

impl<'bytes> Crl<'bytes> {
  /// From PEM contents or in other words, from base64 data delimited by labels.
  ///
  /// See [`crate::misc::Pem`].
  #[cfg(feature = "base64")]
  pub fn from_pem(buffer: &'bytes [u8], pem: &Pem<Range<usize>, 1>) -> crate::Result<Self> {
    let [(_label, range)] = pem.data.as_inner()?;
    Self::decode(&mut GenericDecodeWrapper::new(
      buffer.get(range.clone()).unwrap_or_default(),
      Asn1DecodeWrapper::default(),
    ))
  }

  /// Generalization of [`Self::from_pem`].
  #[cfg(feature = "base64")]
  pub fn from_pems<'pem, I>(
    buffer: &'bytes [u8],
    instances: &mut I,
    pems: impl IntoIterator<Item = &'pem Pem<Range<usize>, 1>>,
  ) -> crate::Result<()>
  where
    I: TryExtend<[Crl<'bytes>; 1]>,
  {
    for pem in pems {
      let [(_label, range)] = pem.data.as_inner()?;
      let bytes = buffer.get(range.clone()).unwrap_or_default();
      instances.try_extend([Self::decode(&mut GenericDecodeWrapper::new(
        bytes,
        Asn1DecodeWrapper::default(),
      ))?])?;
    }
    Ok(())
  }
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
