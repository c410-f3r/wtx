use crate::{
  asn1::{
    Asn1DecodeWrapper, OID_X509_EXT_AUTHORITY_KEY_IDENTIFIER, OID_X509_EXT_BASIC_CONSTRAINTS,
    OID_X509_EXT_CRL_DISTRIBUTION_POINTS, OID_X509_EXT_EXTENDED_KEY_USAGE, OID_X509_EXT_KEY_USAGE,
    OID_X509_EXT_NAME_CONSTRAINTS, OID_X509_EXT_POLICY_CONSTRAINTS, OID_X509_EXT_SUBJECT_ALT_NAME,
    OID_X509_EXT_SUBJECT_KEY_IDENTIFIER,
  },
  codec::{Decode, DecodeWrapper},
  misc::RefOrOwned,
  x509::{
    AlgorithmIdentifier, Certificate, FlaggedExtension, SerialNumber, SubjectPublicKeyInfo,
    TbsCertificate, Validity, X509CvError,
    cv::{validate_ee_static, validate_ica_static},
    extensions::{
      AuthorityKeyIdentifier, BasicConstraints, CrlDistributionPoints, ExtendedKeyUsage, KeyUsage,
      NameConstraints, SubjectAlternativeName, SubjectKeyIdentifier,
    },
    name::NameVector,
  },
};

/// Chain Validation - End Entity
///
/// The final leaf of a PKI chain and also the entry-point where chains can be validated.
///
/// * Servers should concurrently or sequentially call [`Self::validate_chain`],
///   [`Self::validate_signature`] and [`Self::validate_subject_name`] to fully validate
///   certificates.
///
/// * Clients should concurrently or sequentially call [`Self::validate_chain`] and
///   [`Self::validate_signature`] to fully validate certificates.
pub type CvEndEntity<'any, 'bytes> = CvCertificate<'any, 'bytes, true>;

/// Chain Validation - Intermediate
///
/// This certificate is considered an intermediate. There are no entrypoints for chaiCvvalidation.
pub type CvIntermediate<'any, 'bytes> = CvCertificate<'any, 'bytes, false>;

/// Chain Validation - Certificate
///
/// Full X.509 certificates are huge and because of that, this particular structure only has the
/// fields required to perform a chain validation.
#[derive(Debug, PartialEq)]
pub struct CvCertificate<'any, 'bytes, const IS_EE: bool> {
  pub(crate) authority_key_identifier: Option<AuthorityKeyIdentifier>,
  pub(crate) basic_constraints: Option<FlaggedExtension<BasicConstraints>>,
  pub(crate) crl_distribution_points: Option<CrlDistributionPoints<'bytes>>,
  pub(crate) extended_key_usage: Option<FlaggedExtension<ExtendedKeyUsage>>,
  pub(crate) has_unknown_critical_extension: bool,
  pub(crate) is_self_signed: bool,
  pub(crate) issuer: RefOrOwned<'any, NameVector<'bytes>>,
  pub(crate) key_usage: Option<KeyUsage>,
  pub(crate) name_constraints: Option<NameConstraints<'bytes>>,
  pub(crate) serial: SerialNumber,
  pub(crate) signature: &'bytes [u8],
  pub(crate) signature_algorithm: RefOrOwned<'any, AlgorithmIdentifier<'bytes>>,
  pub(crate) signature_msg: &'bytes [u8],
  pub(crate) subject_public_key_info: RefOrOwned<'any, SubjectPublicKeyInfo<'bytes>>,
  pub(crate) subject: RefOrOwned<'any, NameVector<'bytes>>,
  pub(crate) subject_alternative_name: Option<FlaggedExtension<SubjectAlternativeName<'bytes>>>,
  pub(crate) subject_key_identifier: Option<FlaggedExtension<SubjectKeyIdentifier>>,
  pub(crate) validity: Validity,
}

impl<'any, 'bytes, const IS_EE: bool> TryFrom<Certificate<'bytes>>
  for CvCertificate<'any, 'bytes, IS_EE>
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: Certificate<'bytes>) -> Result<Self, Self::Error> {
    let (signature_algorithm, signature_value, tbs) = value.into_parts();
    let parts = Parts::new::<IS_EE>(&tbs)?;
    Ok(Self {
      authority_key_identifier: parts.authority_key_identifier,
      basic_constraints: parts.basic_constraints,
      crl_distribution_points: parts.crl_distribution_points,
      extended_key_usage: parts.extended_key_usage,
      has_unknown_critical_extension: parts.has_unknown_critical_extension,
      is_self_signed: parts.is_self_signed,
      issuer: RefOrOwned::Right(tbs.issuer),
      key_usage: parts.key_usage,
      name_constraints: parts.name_constraints,
      serial: tbs.serial_number.clone(),
      signature_algorithm: RefOrOwned::Right(signature_algorithm),
      signature_msg: tbs.bytes,
      signature: signature_value.bytes(),
      subject_key_identifier: parts.subject_key_identifier,
      subject_public_key_info: RefOrOwned::Right(tbs.subject_public_key_info),
      subject: RefOrOwned::Right(tbs.subject),
      subject_alternative_name: parts.subject_alt_name,
      validity: tbs.validity.clone(),
    })
  }
}

impl<'any, 'bytes, const IS_EE: bool> TryFrom<&'any Certificate<'bytes>>
  for CvCertificate<'any, 'bytes, IS_EE>
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &'any Certificate<'bytes>) -> Result<Self, Self::Error> {
    let tbs = value.tbs_certificate();
    let parts = Parts::new::<IS_EE>(tbs)?;
    Ok(Self {
      authority_key_identifier: parts.authority_key_identifier,
      basic_constraints: parts.basic_constraints,
      crl_distribution_points: parts.crl_distribution_points,
      extended_key_usage: parts.extended_key_usage,
      has_unknown_critical_extension: parts.has_unknown_critical_extension,
      is_self_signed: tbs.issuer == tbs.subject,
      issuer: RefOrOwned::Left(&tbs.issuer),
      key_usage: parts.key_usage,
      name_constraints: parts.name_constraints,
      serial: tbs.serial_number.clone(),
      signature_algorithm: RefOrOwned::Left(value.signature_algorithm()),
      signature_msg: tbs.bytes,
      signature: value.signature_value().bytes(),
      subject_public_key_info: RefOrOwned::Left(&tbs.subject_public_key_info),
      subject: RefOrOwned::Left(&tbs.subject),
      subject_key_identifier: parts.subject_key_identifier,
      subject_alternative_name: parts.subject_alt_name,
      validity: tbs.validity.clone(),
    })
  }
}

struct Parts<'bytes> {
  authority_key_identifier: Option<AuthorityKeyIdentifier>,
  basic_constraints: Option<FlaggedExtension<BasicConstraints>>,
  crl_distribution_points: Option<CrlDistributionPoints<'bytes>>,
  extended_key_usage: Option<FlaggedExtension<ExtendedKeyUsage>>,
  has_unknown_critical_extension: bool,
  is_self_signed: bool,
  key_usage: Option<KeyUsage>,
  name_constraints: Option<NameConstraints<'bytes>>,
  subject_alt_name: Option<FlaggedExtension<SubjectAlternativeName<'bytes>>>,
  subject_key_identifier: Option<FlaggedExtension<SubjectKeyIdentifier>>,
}

impl<'bytes> Parts<'bytes> {
  fn new<const IS_EE: bool>(tbs: &TbsCertificate<'bytes>) -> crate::Result<Self> {
    macro_rules! check_duplicated {
      ($has_duplicated:expr, $opt:expr) => {
        if $opt.is_some() {
          $has_duplicated = true;
          break;
        }
      };
    }

    let is_self_signed = tbs.issuer == tbs.subject;
    let mut authority_key_identifier = None;
    let mut basic_constraints = None;
    let mut crl_distribution_points = None;
    let mut extended_key_usage = None;
    let mut has_unknown_critical_extension = false;
    let mut key_usage = None;
    let mut name_constraints = None;
    let mut subject_alternative_name = None;
    let mut subject_key_identifier = None;

    if let Some(extensions) = tbs.extensions.as_ref() {
      let mut has_duplicated = false;
      for extension in &extensions.entries {
        let decode_aux = Asn1DecodeWrapper::default();
        let mut dw = DecodeWrapper::new(extension.extn_value.bytes(), decode_aux);
        let mut _policy_constraints: Option<()> = None;
        match extension.extn_id {
          el if el == OID_X509_EXT_AUTHORITY_KEY_IDENTIFIER => {
            check_duplicated!(has_duplicated, authority_key_identifier);
            authority_key_identifier = Some(AuthorityKeyIdentifier::decode(&mut dw)?)
          }
          el if el == OID_X509_EXT_BASIC_CONSTRAINTS => {
            check_duplicated!(has_duplicated, basic_constraints);
            basic_constraints =
              Some(FlaggedExtension::new(BasicConstraints::decode(&mut dw)?, extension.critical));
          }
          el if el == OID_X509_EXT_CRL_DISTRIBUTION_POINTS => {
            check_duplicated!(has_duplicated, crl_distribution_points);
            crl_distribution_points = Some(CrlDistributionPoints::decode(&mut dw)?);
          }
          el if el == OID_X509_EXT_EXTENDED_KEY_USAGE => {
            check_duplicated!(has_duplicated, extended_key_usage);
            extended_key_usage =
              Some(FlaggedExtension::new(ExtendedKeyUsage::decode(&mut dw)?, extension.critical));
          }
          el if el == OID_X509_EXT_KEY_USAGE => {
            check_duplicated!(has_duplicated, key_usage);
            key_usage = Some(KeyUsage::decode(&mut dw)?);
          }
          el if el == OID_X509_EXT_NAME_CONSTRAINTS => {
            check_duplicated!(has_duplicated, name_constraints);
            name_constraints = Some(NameConstraints::decode(&mut dw)?);
          }
          el if el == OID_X509_EXT_POLICY_CONSTRAINTS => {
            check_duplicated!(has_duplicated, _policy_constraints);
            _policy_constraints = Some(());
            if !extension.critical {
              return Err(X509CvError::PolicyConstraintMustBeCritical.into());
            }
          }
          el if el == OID_X509_EXT_SUBJECT_KEY_IDENTIFIER => {
            check_duplicated!(has_duplicated, subject_key_identifier);
            subject_key_identifier = Some(FlaggedExtension::new(
              SubjectKeyIdentifier::decode(&mut dw)?,
              extension.critical,
            ));
          }
          el if el == OID_X509_EXT_SUBJECT_ALT_NAME => {
            check_duplicated!(has_duplicated, subject_alternative_name);
            subject_alternative_name = Some(FlaggedExtension::new(
              SubjectAlternativeName::decode(&mut dw)?,
              extension.critical,
            ));
          }
          _ => {
            has_unknown_critical_extension |= extension.critical;
          }
        }
      }
      if has_duplicated {
        return Err(X509CvError::CertCanNotHaveDuplicateExtensions.into());
      }
    }

    let mut last_err = None;
    if IS_EE {
      let _ = validate_ee_static(basic_constraints, key_usage, &mut last_err, &name_constraints);
    } else {
      let _ = validate_ica_static(basic_constraints, &mut last_err, &tbs.subject);
    }
    if let Some(err) = last_err {
      return Err(err.into());
    }

    Ok(Self {
      authority_key_identifier,
      basic_constraints,
      crl_distribution_points,
      extended_key_usage,
      has_unknown_critical_extension,
      is_self_signed,
      key_usage,
      name_constraints,
      subject_alt_name: subject_alternative_name,
      subject_key_identifier,
    })
  }
}
