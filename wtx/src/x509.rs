//! Implementation of <https://datatracker.ietf.org/doc/html/rfc5280>.
//!
//! Only supports X.509 v3.

mod access_description;
mod algorithm_identifier;
mod attribute;
mod attribute_type_and_value;
mod certificate;
mod crl;
mod crl_reason;
mod cv;
mod distribution_point_name;
mod extension;
pub mod extensions;
mod flagged_extension;
mod general_name;
mod general_subtree;
mod ip_address;
mod key_identifier;
mod name;
mod opt_time;
mod reason_flags;
mod relative_distinguished_name;
mod revoked_certificate;
mod revoked_certificates;
mod rsassa_pss_params;
mod serial_number;
mod server_name;
mod subject_public_key_info;
mod tav_store;
mod tbs_cert_list;
mod tbs_certificate;
mod time;
mod validity;
mod verified_path;
mod x509_cv_error;
mod x509_error;

pub use access_description::AccessDescription;
pub use algorithm_identifier::AlgorithmIdentifier;
pub use attribute::Attribute;
pub use attribute_type_and_value::AttributeTypeAndValue;
pub use certificate::Certificate;
pub use crl::Crl;
pub use crl_reason::CrlReason;
pub use cv::{
  cv_certificate::{CvCertificate, CvEndEntity, CvIntermediate},
  cv_crl::CvCrl,
  cv_crl_expiration::CvCrlExpiration,
  cv_evaluation_depth::CvEvaluationDepth,
  cv_policy::CvPolicy,
  cv_policy_mode::CvPolicyMode,
  cv_revoked_certificate::CvRevokedCertificate,
  cv_trust_anchor::CvTrustAnchor,
};
pub use distribution_point_name::DistributionPointName;
pub use extension::Extension;
pub use extensions::Extensions;
pub use flagged_extension::FlaggedExtension;
pub use general_name::{GeneralName, GeneralNames};
pub use general_subtree::GeneralSubtree;
pub use ip_address::IpAddress;
pub use key_identifier::KeyIdentifier;
pub use name::{Name, NameArrayVector, NameVector};
pub use opt_time::OptTime;
pub use reason_flags::ReasonFlags;
pub use relative_distinguished_name::RelativeDistinguishedName;
pub use revoked_certificate::RevokedCertificate;
pub use revoked_certificates::RevokedCertificates;
pub use rsassa_pss_params::RsassaPssParams;
pub use serial_number::SerialNumber;
pub use server_name::ServerName;
pub use subject_public_key_info::SubjectPublicKeyInfo;
pub use tav_store::TavStore;
pub use tbs_cert_list::TbsCertList;
pub use tbs_certificate::TbsCertificate;
pub use time::Time;
pub use validity::Validity;
pub use verified_path::VerifiedPath;
pub use x509_cv_error::X509CvError;
pub use x509_error::X509Error;

/// The maximum number of intermediates between the end entity and the trust anchor.
pub const MAX_INTERMEDIATES: usize = 8;

// Explicit
const EXPLICIT_TAG0: u8 = 160;
const EXPLICIT_TAG3: u8 = 163;

// Implicit
const AUTHORITY_CERT_ISSUER_TAG: u8 = 161;
const AUTHORITY_CERT_SERIAL_NUMBER_TAG: u8 = 130;
const CRL_ISSUER_TAG: u8 = 162;
const DISTRIBUTION_POINT_NAME_FULL_NAME_TAG: u8 = 160;
const DISTRIBUTION_POINT_NAME_RELATIVE_TAG: u8 = 161;
const DISTRIBUTION_POINT_TAG: u8 = 160;
const EXCLUDED_SUBTREES_TAG: u8 = 161;
const INDIRECT_CRL_TAG: u8 = 132;
const INHIBIT_POLICY_MAPPING_TAG: u8 = 129;
const ISSUER_UID_TAG: u8 = 129;
const KEY_IDENTIFIER_TAG: u8 = 128;
const ONLY_CONTAINS_ATTRIBUTE_CERTS_TAG: u8 = 133;
const ONLY_CONTAINS_CA_CERTS_TAG: u8 = 130;
const ONLY_CONTAINS_USER_CERTS_TAG: u8 = 129;
const ONLY_SOME_REASONS_TAG: u8 = 131;
const PERMITTED_SUBTREES_TAG: u8 = 160;
const REASONS_TAG: u8 = 129;
const REQUIRE_EXPLICIT_POLICY_TAG: u8 = 128;
const SUBJECT_UID_TAG: u8 = 130;
