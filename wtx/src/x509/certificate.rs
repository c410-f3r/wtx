use crate::{
  asn1::{BitString, Len, SEQUENCE_TAG, asn1_writer, decode_asn1_tlv},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::{AlgorithmIdentifier, TbsCertificate, X509Error},
};
#[cfg(feature = "base64")]
use crate::{collection::TryExtend, misc::LeaseMut};

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

  /// From PEM contents or in other words, from base64 data.
  ///
  /// See [`crate::misc::Pem`].
  #[cfg(feature = "base64")]
  pub fn from_pem<B>(buffer: &'bytes mut B, bytes: &[u8]) -> crate::Result<Self>
  where
    B: LeaseMut<[u8]> + TryExtend<(u8, usize)>,
  {
    use crate::{
      codec::{Decode, GenericDecodeWrapper},
      misc::Pem,
    };
    let pem = Pem::<_, 1>::decode(&mut GenericDecodeWrapper::new(bytes, &mut *buffer))?;
    let [(_label, _content)] = pem.data.into_inner()?;
    Self::decode(&mut GenericDecodeWrapper::new(buffer.lease(), None))
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
