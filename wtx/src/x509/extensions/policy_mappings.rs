use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, Len, Oid, SEQUENCE_TAG, SequenceBuffer, asn1_writer,
    decode_asn1_tlv,
  },
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  collection::ArrayVectorU8,
  x509::X509Error,
};

/// A single policy mapping entry.
#[derive(Debug, PartialEq)]
pub struct PolicyMapping {
  /// The policy OID in the issuer's domain.
  pub issuer_domain_policy: Oid,
  /// The equivalent policy OID in the subject's domain.
  pub subject_domain_policy: Oid,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for PolicyMapping {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidExtensionPolicyMappings.into());
    };
    dw.bytes = value;
    let issuer_domain_policy = Oid::decode(dw)?;
    let subject_domain_policy = Oid::decode(dw)?;
    dw.bytes = rest;
    Ok(Self { issuer_domain_policy, subject_domain_policy })
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for PolicyMapping {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_ONE_BYTE, SEQUENCE_TAG, |local_ew| {
      self.issuer_domain_policy.encode(local_ew)?;
      self.subject_domain_policy.encode(local_ew)?;
      Ok(())
    })
  }
}

/// It lists one or more pairs of OIDs; each pair includes an issuerDomainPolicy and a
/// subjectDomainPolicy. The pairing indicates the issuing CA considers its
/// issuerDomainPolicy equivalent to the subject CA's subjectDomainPolicy.
#[derive(Debug, PartialEq)]
pub struct PolicyMappings(
  /// Policy mapping entries.
  pub ArrayVectorU8<PolicyMapping, 2>,
);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for PolicyMappings {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Ok(Self(SequenceBuffer::decode(dw, SEQUENCE_TAG)?.0))
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for PolicyMappings {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    SequenceBuffer(&self.0).encode(ew, Len::MAX_ONE_BYTE, SEQUENCE_TAG)
  }
}
