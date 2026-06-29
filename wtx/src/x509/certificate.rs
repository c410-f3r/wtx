use crate::{
  asn1::{
    Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, BitString, Len, SEQUENCE_TAG, asn1_writer,
    decode_asn1_tlv, parse_der_from_pem_range,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collections::TryExtend,
  misc::{Lease, LeaseMut, Pem},
  x509::{AlgorithmIdentifier, TbsCertificate, X509CvError, X509Error},
};

/// A complete X.509 certificate comprising the signed data, algorithm, and signature.
#[derive(Debug, PartialEq)]
pub struct Certificate<B>
where
  B: Lease<[u8]>,
{
  /// See [`AlgorithmIdentifier`].
  signature_algorithm: AlgorithmIdentifier<B>,
  /// The digital signature computed over the DER encoding of [`TbsCertificate`].
  signature_value: BitString<B>,
  /// See [`TbsCertificate`].
  tbs_certificate: TbsCertificate<B>,
}

impl<'this, B> Certificate<B>
where
  B: Lease<[u8]>,
{
  /// Does basic validation like signature mismatch.
  #[inline]
  pub fn new(
    signature_algorithm: AlgorithmIdentifier<B>,
    signature_value: BitString<B>,
    tbs_certificate: TbsCertificate<B>,
  ) -> crate::Result<Self> {
    if signature_algorithm.algorithm != tbs_certificate.signature.algorithm {
      return Err(X509CvError::CertificateAlgorithmMismatch.into());
    }
    Ok(Self { signature_algorithm, signature_value, tbs_certificate })
  }

  /// From DER data
  #[inline]
  pub fn from_der(bytes: &'this [u8]) -> crate::Result<Self>
  where
    B: TryFrom<&'this [u8]>,
    <B as TryFrom<&'this [u8]>>::Error: Into<crate::Error>,
  {
    Self::decode(&mut DecodeWrapper::new(bytes, Asn1DecodeWrapperAux::default()))
  }

  /// From PEM data
  #[inline]
  pub fn from_pem<BUF>(buffer: &'this mut BUF, bytes: &[u8]) -> crate::Result<(Self, &'this [u8])>
  where
    B: TryFrom<&'this [u8]>,
    BUF: LeaseMut<[u8]> + TryExtend<(u8, usize)>,
    <B as TryFrom<&'this [u8]>>::Error: Into<crate::Error>,
  {
    let pem = Pem::decode(&mut DecodeWrapper::new(bytes, &mut *buffer))?;
    let slice: &'this [u8] = <BUF as Lease<[u8]>>::lease(buffer);
    parse_der_from_pem_range(slice, &pem)
  }

  /// Returns the inner elements
  #[inline]
  pub fn into_parts(self) -> (AlgorithmIdentifier<B>, BitString<B>, TbsCertificate<B>) {
    (self.signature_algorithm, self.signature_value, self.tbs_certificate)
  }

  /// See [`AlgorithmIdentifier`].
  #[inline]
  pub const fn signature_algorithm(&self) -> &AlgorithmIdentifier<B> {
    &self.signature_algorithm
  }

  /// Signature derived from the bytes of [`TbsCertificate`].
  #[inline]
  pub const fn signature_value(&self) -> &BitString<B> {
    &self.signature_value
  }

  /// See [`TbsCertificate`].
  #[inline]
  pub const fn tbs_certificate(&self) -> &TbsCertificate<B> {
    &self.tbs_certificate
  }
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for Certificate<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, len, value, []) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidCertificate.into());
    };
    dw.bytes = value;
    dw.decode_aux.inc_curr_idx(len.bytes().len().wrapping_add(1).into());
    let tbs_certificate = TbsCertificate::decode(dw)?;
    let signature_algorithm = AlgorithmIdentifier::decode(dw)?;
    let signature_value = BitString::decode(dw)?;
    Self::new(signature_algorithm, signature_value, tbs_certificate)
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for Certificate<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
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
  use crate::{collections::Vector, x509::Certificate};

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
    drop(Certificate::<&[u8]>::from_pem(&mut Vector::new(), pem.as_bytes()).unwrap());
  }
}
