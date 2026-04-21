use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, BitString, Len, SEQUENCE_TAG, asn1_writer,
    decode_asn1_tlv, parse_der_from_pem_range,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collection::TryExtend,
  misc::{LeaseMut, Pem},
  x509::{AlgorithmIdentifier, TbsCertificate, X509CvError, X509Error},
};

/// A complete X.509 certificate comprising the signed data, algorithm, and signature.
#[derive(Debug, PartialEq)]
pub struct Certificate<'bytes> {
  /// See [`AlgorithmIdentifier`].
  signature_algorithm: AlgorithmIdentifier<'bytes>,
  /// The digital signature computed over the DER encoding of tbs_certificate.
  signature_value: BitString<&'bytes [u8]>,
  /// See [`TbsCertificate`].
  tbs_certificate: TbsCertificate<'bytes>,
}

impl<'bytes> Certificate<'bytes> {
  /// Does basic validation like signature mismatch.
  #[inline]
  pub fn new(
    signature_algorithm: AlgorithmIdentifier<'bytes>,
    signature_value: BitString<&'bytes [u8]>,
    tbs_certificate: TbsCertificate<'bytes>,
  ) -> crate::Result<Self> {
    if signature_algorithm.algorithm != tbs_certificate.signature.algorithm {
      return Err(X509CvError::CertificateAlgorithmMismatch.into());
    }
    Ok(Self { signature_algorithm, signature_value, tbs_certificate })
  }

  /// From DER data
  #[inline]
  pub fn from_der(bytes: &'bytes [u8]) -> crate::Result<Self> {
    Self::decode(&mut DecodeWrapper::new(bytes, Asn1DecodeWrapper::default()))
  }

  /// From PEM data
  #[inline]
  pub fn from_pem<B>(buffer: &'bytes mut B, bytes: &[u8]) -> crate::Result<Self>
  where
    B: LeaseMut<[u8]> + TryExtend<(u8, usize)>,
  {
    let pem = Pem::decode(&mut DecodeWrapper::new(bytes, &mut *buffer))?;
    parse_der_from_pem_range(buffer.lease(), &pem)
  }

  /// Returns the inner elements
  #[inline]
  pub fn into_parts(
    self,
  ) -> (AlgorithmIdentifier<'bytes>, BitString<&'bytes [u8]>, TbsCertificate<'bytes>) {
    (self.signature_algorithm, self.signature_value, self.tbs_certificate)
  }

  /// See [`AlgorithmIdentifier`].
  #[inline]
  pub const fn signature_algorithm(&self) -> &AlgorithmIdentifier<'bytes> {
    &self.signature_algorithm
  }

  /// Signature derived from the bytes of [`TbsCertificate`].
  #[inline]
  pub const fn signature_value(&self) -> &BitString<&'bytes [u8]> {
    &self.signature_value
  }

  /// See [`TbsCertificate`].
  #[inline]
  pub const fn tbs_certificate(&self) -> &TbsCertificate<'bytes> {
    &self.tbs_certificate
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for Certificate<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidCertificate.into());
    };
    dw.bytes = value;
    let tbs_certificate = TbsCertificate::decode(dw)?;
    let signature_algorithm = AlgorithmIdentifier::decode(dw)?;
    let signature_value = BitString::decode(dw)?;
    dw.bytes = rest;
    Self::new(signature_algorithm, signature_value, tbs_certificate)
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for Certificate<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_THREE_BYTES, SEQUENCE_TAG, |local_ew| {
      self.tbs_certificate.encode(local_ew)?;
      self.signature_algorithm.encode(local_ew)?;
      self.signature_value.encode(local_ew)?;
      Ok(())
    })
  }
}

#[cfg(test)]
mod tests {
  use crate::{collection::Vector, x509::Certificate};

  #[test]
  fn empty_sequence() {
    let pem = "-----BEGIN CERTIFICATE-----\
    \nMIIBrDCCAVOgAwIBAgIUb8QzZKLoeb7sNgiYOnbSUl5b4X8wCgYIKoZIzj0EAwIw\n\
    GjEYMBYGA1UEAwwPSW50ZXJtZWRpYXRlIEExMCAXDTcwMDEwMTAwMDAwMVoYDzI5\n\
    NjkwNTAzMDAwMDAxWjAAMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAESedMtWvD\n\
    23rhzERvRSfeBeQFqpeRDjcH4K9ViX3DgDF9oXzLUs9xz+URdSHSNXxej1u6HvrR\n\
    payAj2wt5InjfaOBjjCBizAdBgNVHQ4EFgQUksJLo1u9/CruvoKUEU6w8su70wIw\n\
    HwYDVR0jBBgwFoAUboFzr4EtV7K3tnIFqOlNYZ6JWf8wCwYDVR0PBAQDAgeAMBMG\n\
    A1UdJQQMMAoGCCsGAQUFBwMBMCcGA1UdEQEB/wQdMBuCGWN2ZS0yMDI0LTA1Njcu\n\
    ZXhhbXBsZS5jb20wCgYIKoZIzj0EAwIDRwAwRAIgS9iooj3BeyKGWamWBmjt1Sou\n\
    GsT1IxNxAG6MSRj8vXkCIA6hk7SbTgKaaF0MvHzE8kOyIHivtVXv63XwyC3326R0\n\
    -----END CERTIFICATE-----\n";
    drop(Certificate::from_pem(&mut Vector::new(), pem.as_bytes()).unwrap());
  }
}
