use crate::{
  asn1::{BitString, Len, SEQUENCE_TAG, asn1_writer, decode_asn1_tlv},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  misc::Lease,
  x509::{AlgorithmIdentifier, TbsCertificate, X509Error},
};
#[cfg(feature = "base64")]
use crate::{collection::TryExtend, misc::Pem};
#[cfg(feature = "base64")]
use core::ops::Range;

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
    Self::decode(&mut GenericDecodeWrapper::new(bytes, None))
  }

  /// From PEM contents or in other words, from base64 data delimited by labels.
  ///
  /// See [`crate::misc::Pem`].
  #[cfg(feature = "base64")]
  pub fn from_pem(buffer: &'bytes [u8], pem: &Pem<Range<usize>, 1>) -> crate::Result<Self> {
    let [(_label, range)] = pem.data.as_inner()?;
    Self::decode(&mut GenericDecodeWrapper::new(
      buffer.get(range.clone()).unwrap_or_default(),
      None,
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
    I: TryExtend<[Certificate<'bytes>; 1]>,
  {
    for pem in pems {
      let [(_label, range)] = pem.data.as_inner()?;
      let bytes = buffer.get(range.clone()).unwrap_or_default();
      let instance = Self::decode(&mut GenericDecodeWrapper::new(bytes, None))?;
      instances.try_extend([instance])?;
    }
    Ok(())
  }
}

impl<'de> Decode<'de, GenericCodec<Option<u8>, ()>> for Certificate<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Option<u8>>) -> crate::Result<Self> {
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

impl<'bytes> Encode<GenericCodec<(), ()>> for Certificate<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, ()>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_THREE, SEQUENCE_TAG, |local_ew| {
      self.tbs_certificate.encode(local_ew)?;
      self.signature_algorithm.encode(local_ew)?;
      self.signature_value.encode(local_ew)?;
      Ok(())
    })
  }
}

impl<'bytes> Lease<Certificate<'bytes>> for Certificate<'bytes> {
  #[inline]
  fn lease(&self) -> &Certificate<'bytes> {
    self
  }
}
