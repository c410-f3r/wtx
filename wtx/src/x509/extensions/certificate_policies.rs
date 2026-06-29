use crate::{
  asn1::{
    Any, Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Len, Oid, Opt, SEQUENCE_TAG, SequenceBuffer,
    asn1_writer, decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collections::ArrayVectorU8,
  misc::Lease,
  x509::X509Error,
};

/// In an end entity certificate, these policy information terms indicate the policy under which
/// the certificate has been issued and the purposes for which the certificate may be used.
#[derive(Debug, PartialEq)]
pub struct CertificatePolicies<B>(
  /// Policy information entries.
  pub ArrayVectorU8<PolicyInformation<B>, 2>,
);

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for CertificatePolicies<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    Ok(Self(SequenceBuffer::decode(dw, SEQUENCE_TAG)?.0.0))
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for CertificatePolicies<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    SequenceBuffer(&self.0).encode(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG)
  }
}

/// A single policy information entry within the certificate policies extension.
#[derive(Debug, PartialEq)]
pub struct PolicyInformation<B> {
  /// Policy identifier.
  pub policy_identifier: Oid,
  /// Optional DER-encoded policy qualifiers sequence.
  pub policy_qualifiers: Option<ArrayVectorU8<PolicyQualifierInfo<B>, 1>>,
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for PolicyInformation<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidExtensionCertificatePolicies.into());
    };
    dw.bytes = value;
    let policy_identifier = Oid::decode(dw)?;
    let policy_qualifiers = Opt::decode_seq(dw, SEQUENCE_TAG)?.0;
    dw.bytes = rest;
    Ok(Self { policy_identifier, policy_qualifiers })
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for PolicyInformation<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG, |local_ew| {
      self.policy_identifier.encode(local_ew)?;
      Opt(&self.policy_qualifiers).encode_seq(local_ew, Len::MAX_ONE_BYTE, SEQUENCE_TAG)?;
      Ok(())
    })
  }
}

/// A policy qualifier for certificate policies.
#[derive(Debug, PartialEq)]
pub struct PolicyQualifierInfo<B> {
  /// See [`Oid`].
  pub policy_qualifier_id: Oid,
  /// Opaque bytes
  pub qualifier: Any<B>,
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for PolicyQualifierInfo<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidExtensionCertificatePolicies.into());
    };
    dw.bytes = value;
    let policy_qualifier_id = Oid::decode(dw)?;
    let qualifier = Any::decode(dw)?;
    dw.bytes = rest;
    Ok(Self { policy_qualifier_id, qualifier })
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for PolicyQualifierInfo<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG, |local_ew| {
      self.policy_qualifier_id.encode(local_ew)?;
      self.qualifier.encode(local_ew)?;
      Ok(())
    })
  }
}
