//! Metadata intended for Certificates or CRLs.

mod authority_information_access;
mod authority_key_identifier;
mod basic_constraints;
mod certificate_issuer;
mod certificate_policies;
mod crl_distribution_points;
mod crl_number;
mod extended_key_usage;
mod freshest_crl;
mod inhibit_any_policy;
mod invalidity_date;
mod issuer_alternative_name;
mod issuing_distribution_point;
mod key_usage;
mod name_constraints;
mod policy_constraints;
mod policy_mappings;
mod reason_code;
mod signed_certificate_timestamp_list;
mod subject_alternative_name;
mod subject_directory_attributes;
mod subject_information_access;
mod subject_key_identifier;
mod transparency_information_syntax;

pub use authority_information_access::AuthorityInformationAccess;
pub use authority_key_identifier::AuthorityKeyIdentifier;
pub use basic_constraints::BasicConstraints;
pub use certificate_issuer::CertificateIssuer;
pub use certificate_policies::{CertificatePolicies, PolicyInformation, PolicyQualifierInfo};
pub use crl_distribution_points::{CrlDistributionPoints, DistributionPoint};
pub use crl_number::CrlNumber;
pub use extended_key_usage::ExtendedKeyUsage;
pub use freshest_crl::FreshestCrl;
pub use inhibit_any_policy::InhibitAnyPolicy;
pub use invalidity_date::InvalidityDate;
pub use issuer_alternative_name::IssuerAlternativeName;
pub use issuing_distribution_point::IssuingDistributionPoint;
pub use key_usage::KeyUsage;
pub use name_constraints::NameConstraints;
pub use policy_constraints::PolicyConstraints;
pub use policy_mappings::{PolicyMapping, PolicyMappings};
pub use reason_code::ReasonCode;
pub use signed_certificate_timestamp_list::SignedCertificateTimestampList;
pub use subject_alternative_name::SubjectAlternativeName;
pub use subject_directory_attributes::SubjectDirectoryAttributes;
pub use subject_information_access::SubjectInformationAccess;
pub use subject_key_identifier::SubjectKeyIdentifier;
pub use transparency_information_syntax::TransparencyInformationSyntax;

use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, Len, SEQUENCE_TAG, SequenceBuffer, asn1_writer,
    decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collection::Vector,
  x509::{Extension, X509Error},
};

/// List of extensions
#[derive(Debug, PartialEq)]
pub struct Extensions<'bytes> {
  /// Entries
  pub entries: Vector<Extension<'bytes>>,
  /// Tag
  pub tag: u8,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for Extensions<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let tag = dw.decode_aux.tag.unwrap_or(SEQUENCE_TAG);
    dw.decode_aux.tag = None;
    let entries = if tag == SEQUENCE_TAG {
      SequenceBuffer::decode(dw, SEQUENCE_TAG)?.0
    } else {
      let (actual_tag, _, value, rest) = decode_asn1_tlv(dw.bytes)?;
      if actual_tag != tag {
        return Err(X509Error::InvalidExtensions(actual_tag).into());
      }
      dw.bytes = value;
      let entries = SequenceBuffer::decode(dw, SEQUENCE_TAG)?.0;
      dw.bytes = rest;
      entries
    };
    Ok(Self { entries, tag })
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for Extensions<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    if self.tag == SEQUENCE_TAG {
      SequenceBuffer(&self.entries).encode(ew, Len::MAX_THREE_BYTES, SEQUENCE_TAG)
    } else {
      asn1_writer(ew, Len::MAX_THREE_BYTES, self.tag, |local_ew| {
        SequenceBuffer(&self.entries).encode(local_ew, Len::MAX_THREE_BYTES, SEQUENCE_TAG)?;
        Ok(())
      })
    }
  }
}
