use crate::{
  asn1::{
    Asn1DecodeWrapper, OID_X509_EXT_AUTHORITY_KEY_IDENTIFIER, OID_X509_EXT_BASIC_CONSTRAINTS,
    OID_X509_EXT_KEY_USAGE, OID_X509_EXT_NAME_CONSTRAINTS, OID_X509_EXT_SUBJECT_KEY_IDENTIFIER,
  },
  codec::{Decode, DecodeWrapper},
  x509::{
    Certificate, FlaggedExtension, SubjectPublicKeyInfo, TbsCertificate, Validity, X509CvError,
    cv::validate_ica_static,
    extensions::{
      AuthorityKeyIdentifier, BasicConstraints, KeyUsage, NameConstraints, SubjectKeyIdentifier,
    },
  },
};

#[cfg(feature = "ccadb")]
pub(crate) type CvTrustAnchorRaw<'bytes> = (
  Option<[u8; 20]>,
  bool,
  bool,
  Option<(u8, u8)>,
  &'bytes [u8],
  Option<([u8; 20], bool)>,
  (&'bytes [u8], Option<(&'bytes [u8], u8)>, &'bytes [u8]),
  (i64, i64),
);

/// Chain Validation - Trust Anchor
///
/// A trust anchor is the top-most certificate in a certificate chain.
///
/// ```ignore
/// Trust Anchors <- Intermediates <- End Entity
/// ```
#[derive(Debug, PartialEq)]
pub struct CvTrustAnchor<'bytes> {
  authority_key_identifier: Option<AuthorityKeyIdentifier>,
  has_unknown_critical_extension: bool,
  is_self_signed: bool,
  key_usage: Option<KeyUsage>,
  name_constraints: Option<NameConstraints<'bytes>>,
  subject: &'bytes [u8],
  subject_key_identifier: Option<FlaggedExtension<SubjectKeyIdentifier>>,
  subject_public_key_info: SubjectPublicKeyInfo<'bytes>,
  validity: Validity,
}

impl<'bytes> CvTrustAnchor<'bytes> {
  /// This constructor doesn't perform checks that assert correctness.
  #[inline]
  pub const fn new(
    authority_key_identifier: Option<AuthorityKeyIdentifier>,
    has_unknown_critical_extension: bool,
    is_self_signed: bool,
    key_usage: Option<KeyUsage>,
    name_constraints: Option<NameConstraints<'bytes>>,
    subject: &'bytes [u8],
    subject_key_identifier: Option<FlaggedExtension<SubjectKeyIdentifier>>,
    subject_public_key_info: SubjectPublicKeyInfo<'bytes>,
    validity: Validity,
  ) -> Self {
    Self {
      authority_key_identifier,
      has_unknown_critical_extension,
      is_self_signed,
      key_usage,
      name_constraints,
      subject,
      subject_key_identifier,
      subject_public_key_info,
      validity,
    }
  }

  #[cfg(feature = "ccadb")]
  pub(crate) fn _from_raw(raw: CvTrustAnchorRaw<'bytes>) -> Option<Self> {
    use crate::{
      asn1::{Any, BitString, Len, Oid},
      calendar::DateTime,
      collection::ArrayVectorU8,
      x509::{AlgorithmIdentifier, KeyIdentifier, Time},
    };

    Some(CvTrustAnchor::new(
      raw.0.map(|el| {
        AuthorityKeyIdentifier::new(Some(KeyIdentifier::new(ArrayVectorU8::from_array_u8(el))))
      }),
      raw.1,
      raw.2,
      raw.3.map(|el| KeyUsage::new(el)),
      None,
      raw.4,
      raw.5.map(|el| {
        let ki = KeyIdentifier::new(ArrayVectorU8::from_array_u8(el.0));
        FlaggedExtension::new(SubjectKeyIdentifier::new(ki), el.1)
      }),
      {
        let algorithm = Oid::from_bytes_opt(raw.6.0)?;
        let parameters = raw.6.1.and_then(|(bytes, tag)| {
          let len = Len::from_u8(bytes.len().try_into().ok()?);
          Some(Any::new(bytes, tag, len))
        });
        SubjectPublicKeyInfo::new(
          AlgorithmIdentifier::new(algorithm, parameters),
          BitString::from_bytes(raw.6.2),
        )
      },
      Validity::new(
        Time::new(DateTime::from_timestamp_secs(raw.7.0).ok()?, false),
        Time::new(DateTime::from_timestamp_secs(raw.7.1).ok()?, false),
      ),
    ))
  }

  /// See [`AuthorityKeyIdentifier`].
  #[inline]
  pub const fn authority_key_identifier(&self) -> &Option<AuthorityKeyIdentifier> {
    &self.authority_key_identifier
  }

  /// Custom extensions can not be critical.
  #[inline]
  pub const fn has_unknown_critical_extension(&self) -> bool {
    self.has_unknown_critical_extension
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

  /// Raw bytes of the subject's field
  #[inline]
  pub const fn subject(&self) -> &'bytes [u8] {
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

impl<'bytes> TryFrom<Certificate<'bytes>> for CvTrustAnchor<'bytes> {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: Certificate<'bytes>) -> Result<Self, Self::Error> {
    let (_, _, tbs) = value.into_parts();
    let parts = Parts::new(&tbs)?;
    Ok(Self {
      authority_key_identifier: parts.authority_key_identifier,
      has_unknown_critical_extension: parts.has_unknown_critical_extension,
      is_self_signed: parts.is_self_signed,
      key_usage: parts.key_usage,
      name_constraints: parts.name_constraints,
      subject: tbs.subject.bytes(),
      subject_key_identifier: parts.subject_key_identifier,
      subject_public_key_info: tbs.subject_public_key_info,
      validity: tbs.validity,
    })
  }
}

impl<'any, 'bytes> TryFrom<&'any Certificate<'bytes>> for CvTrustAnchor<'bytes> {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &'any Certificate<'bytes>) -> Result<Self, Self::Error> {
    let tbs = value.tbs_certificate();
    let parts = Parts::new(tbs)?;
    Ok(Self {
      authority_key_identifier: parts.authority_key_identifier,
      has_unknown_critical_extension: parts.has_unknown_critical_extension,
      is_self_signed: parts.is_self_signed,
      key_usage: parts.key_usage,
      name_constraints: parts.name_constraints,
      subject: value.tbs_certificate().subject.bytes(),
      subject_key_identifier: parts.subject_key_identifier,
      subject_public_key_info: value.tbs_certificate().subject_public_key_info.clone(),
      validity: value.tbs_certificate().validity.clone(),
    })
  }
}

struct Parts<'bytes> {
  authority_key_identifier: Option<AuthorityKeyIdentifier>,
  has_unknown_critical_extension: bool,
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

    let is_self_signed = tbs.issuer.bytes() == tbs.subject.bytes();
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
    let _ = validate_ica_static(basic_constraints, &mut last_err, &tbs.subject);
    if let Some(err) = last_err {
      return Err(err.into());
    }

    Ok(Self {
      authority_key_identifier,
      has_unknown_critical_extension,
      is_self_signed,
      key_usage,
      name_constraints,
      subject_key_identifier,
    })
  }
}
