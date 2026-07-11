macro_rules! check_duplicated {
  ($has_duplicated:expr, $opt:expr) => {
    if $opt.is_some() {
      $has_duplicated = true;
      break;
    }
  };
}

use crate::{
  asn1::{
    Asn1DecodeWrapperAux, OID_X509_EXT_AUTHORITY_KEY_IDENTIFIER, OID_X509_EXT_BASIC_CONSTRAINTS,
    OID_X509_EXT_CRL_DISTRIBUTION_POINTS, OID_X509_EXT_EXTENDED_KEY_USAGE, OID_X509_EXT_KEY_USAGE,
    OID_X509_EXT_NAME_CONSTRAINTS, OID_X509_EXT_POLICY_CONSTRAINTS, OID_X509_EXT_SUBJECT_ALT_NAME,
    OID_X509_EXT_SUBJECT_KEY_IDENTIFIER,
  },
  codec::{Decode as _, DecodeWrapper},
  misc::Lease,
  x509::{
    AlgorithmIdentifier, Certificate, FlaggedExtension, Name, SerialNumber, SubjectPublicKeyInfo,
    TbsCertificate, Validity, X509CvError,
    cv::{validate_ee_static, validate_ica_static},
    extensions::{
      AuthorityKeyIdentifier, BasicConstraints, CrlDistributionPoints, ExtendedKeyUsage, KeyUsage,
      NameConstraints, SubjectAlternativeName, SubjectKeyIdentifier,
    },
  },
};

/// Chain Validation - End Entity
///
/// The final leaf of a PKI chain and also the entry-point where chains can be validated.
///
/// * Clients should concurrently or sequentially call [`Self::validate_chain`],
///   `validate_signature` and [`Self::validate_subject_name`] to fully validate
///   ***server*** certificates.
///
/// * Servers should concurrently or sequentially call [`Self::validate_chain`] and
///   `validate_signature` to fully validate ***client*** certificates when mTLS is involved.
pub type CvEndEntity<B> = CvCertificate<B, true>;

/// Chain Validation - Intermediate
///
/// This certificate is considered an intermediate. There are no entrypoints for chain validation.
pub type CvIntermediate<B> = CvCertificate<B, false>;

/// Chain Validation - Certificate
///
/// Full X.509 certificates are huge and because of that, this particular structure only has the
/// fields required to perform a chain validation.
#[derive(Clone, Debug, PartialEq)]
pub struct CvCertificate<B, const IS_EE: bool>
where
  B: Lease<[u8]>,
{
  pub(crate) authority_key_identifier: Option<AuthorityKeyIdentifier>,
  pub(crate) basic_constraints: Option<FlaggedExtension<BasicConstraints>>,
  pub(crate) crl_distribution_points: Option<CrlDistributionPoints<B>>,
  pub(crate) extended_key_usage: Option<FlaggedExtension<ExtendedKeyUsage>>,
  pub(crate) has_unknown_critical_extension: bool,
  pub(crate) is_self_signed: bool,
  pub(crate) issuer: B,
  pub(crate) key_usage: Option<KeyUsage>,
  pub(crate) name_constraints: Option<NameConstraints<B>>,
  pub(crate) serial: SerialNumber,
  pub(crate) signature: B,
  pub(crate) signature_algorithm: AlgorithmIdentifier<B>,
  pub(crate) signature_msg: B,
  pub(crate) subject_public_key_info: SubjectPublicKeyInfo<B>,
  pub(crate) subject: Name<B>,
  pub(crate) subject_alternative_name: Option<FlaggedExtension<SubjectAlternativeName<B>>>,
  pub(crate) subject_key_identifier: Option<FlaggedExtension<SubjectKeyIdentifier>>,
  pub(crate) validity: Validity,
}

impl<B, const IS_EE: bool> CvCertificate<B, IS_EE>
where
  B: Lease<[u8]>,
{
  /// See [`SubjectPublicKeyInfo`].
  #[inline]
  pub const fn subject_public_key_info(&self) -> &SubjectPublicKeyInfo<B> {
    &self.subject_public_key_info
  }
}

impl<'cert, const IS_EE: bool> CvCertificate<&'cert [u8], IS_EE> {
  /// New instances that consumes the originating certificate.
  ///
  /// For verification purposes it is also necessary to provide the signature message that is
  /// composed by the bytes of `TbsCertificate`.
  #[inline]
  pub fn from_certificate(
    certificate: Certificate<&'cert [u8]>,
    signature_msg: &'cert [u8],
  ) -> crate::Result<Self> {
    let (signature_algorithm, signature_value, tbs) = certificate.into_parts();
    let parts = CvCertificateExtensions::new::<IS_EE>(&tbs)?;
    Ok(Self {
      authority_key_identifier: parts.authority_key_identifier,
      basic_constraints: parts.basic_constraints,
      crl_distribution_points: parts.crl_distribution_points,
      extended_key_usage: parts.extended_key_usage,
      has_unknown_critical_extension: parts.has_unknown_critical_extension,
      is_self_signed: parts.is_self_signed,
      issuer: tbs.issuer.bytes(),
      key_usage: parts.key_usage,
      name_constraints: parts.name_constraints,
      serial: tbs.serial_number.clone(),
      signature: signature_value.bytes(),
      signature_algorithm,
      signature_msg,
      subject_key_identifier: parts.subject_key_identifier,
      subject_public_key_info: tbs.subject_public_key_info,
      subject: tbs.subject,
      subject_alternative_name: parts.subject_alt_name,
      validity: tbs.validity.clone(),
    })
  }
}

struct CvCertificateExtensions<B> {
  authority_key_identifier: Option<AuthorityKeyIdentifier>,
  basic_constraints: Option<FlaggedExtension<BasicConstraints>>,
  crl_distribution_points: Option<CrlDistributionPoints<B>>,
  extended_key_usage: Option<FlaggedExtension<ExtendedKeyUsage>>,
  has_unknown_critical_extension: bool,
  is_self_signed: bool,
  key_usage: Option<KeyUsage>,
  name_constraints: Option<NameConstraints<B>>,
  subject_alt_name: Option<FlaggedExtension<SubjectAlternativeName<B>>>,
  subject_key_identifier: Option<FlaggedExtension<SubjectKeyIdentifier>>,
}

impl<'cert> CvCertificateExtensions<&'cert [u8]> {
  fn new<const IS_EE: bool>(tbs: &TbsCertificate<&'cert [u8]>) -> crate::Result<Self> {
    let is_self_signed = tbs.issuer.bytes() == tbs.subject.bytes();
    let mut authority_key_identifier = None;
    let mut basic_constraints = None;
    let mut crl_distribution_points = None;
    let mut extended_key_usage = None;
    let mut has_unknown_critical_extension = false;
    let mut key_usage = None;
    let mut name_constraints = None;
    let mut _policy_constraints: Option<()> = None;
    let mut subject_alt_name = None;
    let mut subject_key_identifier = None;

    if let Some(extensions) = tbs.extensions.as_ref() {
      let mut has_duplicated = false;
      for extension in &extensions.entries {
        let decode_aux = Asn1DecodeWrapperAux::default();
        let mut dw = DecodeWrapper::new(extension.extn_value.bytes(), decode_aux);
        match extension.extn_id {
          el if el == OID_X509_EXT_AUTHORITY_KEY_IDENTIFIER => {
            check_duplicated!(has_duplicated, authority_key_identifier);
            authority_key_identifier = Some(AuthorityKeyIdentifier::decode(&mut dw)?);
          }
          el if el == OID_X509_EXT_BASIC_CONSTRAINTS => {
            check_duplicated!(has_duplicated, basic_constraints);
            basic_constraints =
              Some(FlaggedExtension::new(BasicConstraints::decode(&mut dw)?, extension.critical));
          }
          el if el == OID_X509_EXT_CRL_DISTRIBUTION_POINTS => {
            check_duplicated!(has_duplicated, crl_distribution_points);
            crl_distribution_points = Some(CrlDistributionPoints::<&'cert [u8]>::decode(&mut dw)?);
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
            check_duplicated!(has_duplicated, subject_alt_name);
            subject_alt_name = Some(FlaggedExtension::new(
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
      let _ = validate_ica_static(
        basic_constraints,
        is_self_signed,
        key_usage,
        &mut last_err,
        &tbs.subject,
      );
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
      subject_alt_name,
      subject_key_identifier,
    })
  }
}
