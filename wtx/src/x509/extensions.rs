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
mod subject_alternative_name;
mod subject_directory_attributes;
mod subject_information_access;
mod subject_key_identifier;

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
pub use subject_alternative_name::SubjectAlternativeName;
pub use subject_directory_attributes::SubjectDirectoryAttributes;
pub use subject_information_access::SubjectInformationAccess;
pub use subject_key_identifier::SubjectKeyIdentifier;

use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, Len, SEQUENCE_TAG, SequenceBuffer, asn1_writer,
    decode_asn1_tlv,
  },
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  collection::{TryExtend, Vector},
  x509::{Extension, X509Error},
};

/// List of extensions
#[derive(Debug, PartialEq)]
pub struct Extensions<'bytes, const TAG: u8>(
  /// List of extensions
  pub Vector<Extension<'bytes>>,
);

impl<'de, const TAG: u8> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for Extensions<'de, TAG> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (tag, _, value, rest) = decode_asn1_tlv(dw.bytes)?;
    if tag != TAG {
      return Err(X509Error::InvalidExtensions(tag).into());
    }
    dw.bytes = value;
    let collection = if TAG == SEQUENCE_TAG {
      let mut extensions = Vector::default();
      while !dw.bytes.is_empty() {
        extensions.try_extend([Extension::decode(dw)?])?;
      }
      extensions
    } else {
      SequenceBuffer::decode(dw, SEQUENCE_TAG)?.0
    };
    dw.bytes = rest;
    Ok(Self(collection))
  }
}

impl<'bytes, const TAG: u8> Encode<GenericCodec<(), Asn1EncodeWrapper>>
  for Extensions<'bytes, TAG>
{
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_THREE_BYTES, TAG, |local_ew| {
      if TAG == SEQUENCE_TAG {
        for elem in self.0.iter() {
          elem.encode(local_ew)?;
        }
      } else {
        SequenceBuffer(&self.0).encode(local_ew, Len::MAX_ONE_BYTE, SEQUENCE_TAG)?;
      }
      Ok(())
    })
  }
}
