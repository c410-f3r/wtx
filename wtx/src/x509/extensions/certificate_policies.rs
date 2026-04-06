use crate::{
  asn1::{
    Any, Asn1DecodeWrapper, Asn1EncodeWrapper, Len, Oid, Opt, SEQUENCE_TAG, SequenceBuffer,
    asn1_writer, decode_asn1_tlv,
  },
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  collection::ArrayVectorU8,
  x509::X509Error,
};

/// In an end entity certificate, these policy information terms indicate the policy under which
/// the certificate has been issued and the purposes for which the certificate may be used.
#[derive(Debug, PartialEq)]
pub struct CertificatePolicies<'bytes>(
  /// Policy information entries.
  pub ArrayVectorU8<PolicyInformation<'bytes>, 2>,
);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for CertificatePolicies<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Ok(Self(SequenceBuffer::decode(dw, SEQUENCE_TAG)?.0))
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for CertificatePolicies<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    SequenceBuffer(&self.0).encode(ew, Len::MAX_ONE_BYTE, SEQUENCE_TAG)
  }
}

/// A single policy information entry within the certificate policies extension.
#[derive(Debug, PartialEq)]
pub struct PolicyInformation<'bytes> {
  /// Policy identifier.
  pub policy_identifier: Oid,
  /// Optional DER-encoded policy qualifiers sequence.
  pub policy_qualifiers: Option<ArrayVectorU8<PolicyQualifierInfo<'bytes>, 1>>,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for PolicyInformation<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
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

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for PolicyInformation<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_ONE_BYTE, SEQUENCE_TAG, |local_ew| {
      self.policy_identifier.encode(local_ew)?;
      Opt(&self.policy_qualifiers).encode_seq(local_ew, Len::MAX_ONE_BYTE, SEQUENCE_TAG)?;
      Ok(())
    })
  }
}

/// A policy qualifier for certificate policies.
#[derive(Debug, PartialEq)]
pub struct PolicyQualifierInfo<'bytes> {
  /// See [`Oid`].
  pub policy_qualifier_id: Oid,
  /// Opaque bytes
  pub qualifier: Any<&'bytes [u8]>,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for PolicyQualifierInfo<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
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

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for PolicyQualifierInfo<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_ONE_BYTE, SEQUENCE_TAG, |local_ew| {
      self.policy_qualifier_id.encode(local_ew)?;
      self.qualifier.encode(local_ew)?;
      Ok(())
    })
  }
}
