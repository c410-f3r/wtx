use crate::{
  asn1::{
    Asn1DecodeWrapper, OID_X509_EXT_AUTHORITY_KEY_IDENTIFIER, OID_X509_EXT_BASIC_CONSTRAINTS,
    OID_X509_EXT_KEY_USAGE, OID_X509_EXT_NAME_CONSTRAINTS, OID_X509_EXT_SUBJECT_KEY_IDENTIFIER,
  },
  codec::{Decode, DecodeWrapper},
  misc::RefOrOwned,
  x509::{
    Certificate, FlaggedExtension, NameVector, SubjectPublicKeyInfo, TbsCertificate, Validity,
    X509CvError,
    cv::{validate_common_static, validate_ica_static},
    extensions::{
      AuthorityKeyIdentifier, BasicConstraints, KeyUsage, NameConstraints, SubjectKeyIdentifier,
    },
  },
};

/// Chain Validation - Trust Anchor
///
/// A trust anchor is the top-most certificate in a certificate chain.
///
/// ```ignore
/// Trust Anchors <- Intermediates <- End Entity
/// ```
#[derive(Debug, PartialEq)]
pub struct CvTrustAnchor<'any, 'bytes> {
  authority_key_identifier: Option<AuthorityKeyIdentifier>,
  is_self_signed: bool,
  key_usage: Option<KeyUsage>,
  name_constraints: Option<NameConstraints<'bytes>>,
  subject_key_identifier: Option<FlaggedExtension<SubjectKeyIdentifier>>,
  subject_public_key_info: SubjectPublicKeyInfo<'bytes>,
  subject: RefOrOwned<'any, NameVector<'bytes>>,
  validity: Validity,
}

impl<'any, 'bytes> CvTrustAnchor<'any, 'bytes> {
  /// See [`AuthorityKeyIdentifier`].
  #[inline]
  pub const fn authority_key_identifier(&self) -> &Option<AuthorityKeyIdentifier> {
    &self.authority_key_identifier
  }

  /// If `issuer` is equal to `subject`.
  #[inline]
  pub const fn is_self_signed(&self) -> bool {
    self.is_self_signed
  }

  /// See [`KeyUsage`].
  #[inline]
  pub const fn key_usage(&self) -> Option<KeyUsage> {
    self.key_usage
  }

  /// See [`NameConstraints`]
  #[inline]
  pub const fn name_constraints(&self) -> &Option<NameConstraints<'bytes>> {
    &self.name_constraints
  }

  /// See [`NameVector`].
  #[inline]
  pub const fn subject(&self) -> &RefOrOwned<'any, NameVector<'bytes>> {
    &self.subject
  }

  /// See [`SubjectKeyIdentifier`].
  #[inline]
  pub const fn subject_key_identifier(&self) -> &Option<FlaggedExtension<SubjectKeyIdentifier>> {
    &self.subject_key_identifier
  }

  /// See [`SubjectPublicKeyInfo`].
  #[inline]
  pub const fn subject_public_key_info(&self) -> &SubjectPublicKeyInfo<'bytes> {
    &self.subject_public_key_info
  }

  /// See [`Validity`].
  #[inline]
  pub const fn validity(&self) -> &Validity {
    &self.validity
  }
}

impl<'any, 'bytes> TryFrom<Certificate<'bytes>> for CvTrustAnchor<'any, 'bytes> {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: Certificate<'bytes>) -> Result<Self, Self::Error> {
    let (_, _, tbs) = value.into_parts();
    let parts = Parts::new(&tbs)?;
    Ok(Self {
      authority_key_identifier: parts.authority_key_identifier,
      is_self_signed: parts.is_self_signed,
      key_usage: parts.key_usage,
      name_constraints: parts.name_constraints,
      subject: RefOrOwned::Right(tbs.subject),
      subject_key_identifier: parts.subject_key_identifier,
      subject_public_key_info: tbs.subject_public_key_info,
      validity: tbs.validity,
    })
  }
}

impl<'any, 'bytes> TryFrom<&'any Certificate<'bytes>> for CvTrustAnchor<'any, 'bytes> {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &'any Certificate<'bytes>) -> Result<Self, Self::Error> {
    let tbs = value.tbs_certificate();
    let parts = Parts::new(tbs)?;
    Ok(Self {
      authority_key_identifier: parts.authority_key_identifier,
      is_self_signed: parts.is_self_signed,
      key_usage: parts.key_usage,
      name_constraints: parts.name_constraints,
      subject: RefOrOwned::Left(&value.tbs_certificate().subject),
      subject_key_identifier: parts.subject_key_identifier,
      subject_public_key_info: value.tbs_certificate().subject_public_key_info.clone(),
      validity: value.tbs_certificate().validity.clone(),
    })
  }
}

struct Parts<'bytes> {
  authority_key_identifier: Option<AuthorityKeyIdentifier>,
  is_self_signed: bool,
  key_usage: Option<KeyUsage>,
  name_constraints: Option<NameConstraints<'bytes>>,
  subject_key_identifier: Option<FlaggedExtension<SubjectKeyIdentifier>>,
}

impl<'bytes> Parts<'bytes> {
  fn new(tbs: &TbsCertificate<'bytes>) -> crate::Result<Self> {
    macro_rules! check_duplicated {
      ($has_duplicated:expr, $opt:expr) => {
        if $opt.is_some() {
          $has_duplicated = true;
          break;
        }
      };
    }

    let mut authority_key_identifier = None;
    let mut basic_constraints = None;
    let mut has_unknown_critical_extension = false;
    let mut key_usage = None;
    let mut name_constraints = None;
    let mut subject_key_identifier = None;

    if let Some(extensions) = tbs.extensions.as_ref() {
      let mut has_duplicated = false;
      for extension in &extensions.entries {
        let decode_aux = Asn1DecodeWrapper::default();
        let mut dw = DecodeWrapper::new(extension.extn_value.bytes(), decode_aux);
        match extension.extn_id {
          el if el == OID_X509_EXT_AUTHORITY_KEY_IDENTIFIER => {
            check_duplicated!(has_duplicated, authority_key_identifier);
            if extension.critical {
              return Err(X509CvError::AuthorityKeyIdentifierMustNotBeCritical.into());
            }
            authority_key_identifier = Some(AuthorityKeyIdentifier::decode(&mut dw)?);
          }
          el if el == OID_X509_EXT_KEY_USAGE => {
            check_duplicated!(has_duplicated, key_usage);
            key_usage = Some(KeyUsage::decode(&mut dw)?);
          }
          el if el == OID_X509_EXT_SUBJECT_KEY_IDENTIFIER => {
            check_duplicated!(has_duplicated, subject_key_identifier);
            subject_key_identifier = Some(FlaggedExtension::new(
              SubjectKeyIdentifier::decode(&mut dw)?,
              extension.critical,
            ));
          }
          el if el == OID_X509_EXT_BASIC_CONSTRAINTS => {
            check_duplicated!(has_duplicated, basic_constraints);
            basic_constraints =
              Some(FlaggedExtension::new(BasicConstraints::decode(&mut dw)?, extension.critical));
          }
          el if el == OID_X509_EXT_NAME_CONSTRAINTS => {
            check_duplicated!(has_duplicated, name_constraints);
            if !extension.critical {
              return Err(X509CvError::NameConstraintsMustBeCritical.into());
            }
            name_constraints = Some(NameConstraints::decode(&mut dw)?);
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
    let _ = validate_common_static(
      basic_constraints,
      has_unknown_critical_extension,
      key_usage,
      &mut last_err,
    );
    let _ = validate_ica_static(basic_constraints, &mut last_err, &tbs.subject);
    if let Some(err) = last_err {
      return Err(err.into());
    }

    Ok(Self {
      authority_key_identifier,
      is_self_signed: tbs.issuer == tbs.subject,
      key_usage,
      name_constraints,
      subject_key_identifier,
    })
  }
}
