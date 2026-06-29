macro_rules! check_duplicated {
  ($has_duplicated:expr, $opt:expr) => {
    if $opt.is_some() {
      $has_duplicated = true;
      break;
    }
  };
}

macro_rules! extensions {
  ($tbs:expr, $ext:ident => $bytes_expr:expr) => {{
    let is_self_signed = $tbs.issuer.bytes().lease() == $tbs.subject.bytes().lease();
    let mut authority_key_identifier = None;
    let mut basic_constraints = None;
    let mut has_unknown_critical_extension = false;
    let mut key_usage = None;
    let mut name_constraints = None;
    let mut subject_key_identifier = None;

    if let Some(extensions) = $tbs.extensions.as_ref() {
      let mut has_duplicated = false;
      for $ext in &extensions.entries {
        let decode_aux = Asn1DecodeWrapperAux::default();
        let mut dw = DecodeWrapper::new($bytes_expr, decode_aux);
        match $ext.extn_id {
          el if el == OID_X509_EXT_AUTHORITY_KEY_IDENTIFIER => {
            check_duplicated!(has_duplicated, authority_key_identifier);
            if $ext.critical {
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
            subject_key_identifier =
              Some(FlaggedExtension::new(SubjectKeyIdentifier::decode(&mut dw)?, $ext.critical));
          }
          el if el == OID_X509_EXT_BASIC_CONSTRAINTS => {
            check_duplicated!(has_duplicated, basic_constraints);
            basic_constraints =
              Some(FlaggedExtension::new(BasicConstraints::decode(&mut dw)?, $ext.critical));
          }
          el if el == OID_X509_EXT_NAME_CONSTRAINTS => {
            check_duplicated!(has_duplicated, name_constraints);
            if !$ext.critical {
              return Err(X509CvError::NameConstraintsMustBeCritical.into());
            }
            name_constraints = Some(NameConstraints::decode(&mut dw)?);
          }
          _ => {
            has_unknown_critical_extension |= $ext.critical;
          }
        }
      }
      if has_duplicated {
        return Err(X509CvError::CertCanNotHaveDuplicateExtensions.into());
      }
    }

    let mut last_err = None;
    let _ = validate_ica_static(
      basic_constraints,
      is_self_signed,
      key_usage,
      &mut last_err,
      &$tbs.subject,
    );
    if let Some(err) = last_err {
      return Err(err.into());
    }

    CvTrustAnchorExtensions {
      authority_key_identifier,
      has_unknown_critical_extension,
      is_self_signed,
      key_usage,
      name_constraints,
      subject_key_identifier,
    }
  }};
}

use crate::{
  asn1::{
    Any, Asn1DecodeWrapperAux, BitString, OID_X509_EXT_AUTHORITY_KEY_IDENTIFIER,
    OID_X509_EXT_BASIC_CONSTRAINTS, OID_X509_EXT_KEY_USAGE, OID_X509_EXT_NAME_CONSTRAINTS,
    OID_X509_EXT_SUBJECT_KEY_IDENTIFIER,
  },
  codec::{Decode as _, DecodeWrapper},
  misc::Lease,
  x509::{
    AlgorithmIdentifier, Certificate, FlaggedExtension, SubjectPublicKeyInfo, Validity,
    X509CvError,
    cv::validate_ica_static,
    extensions::{
      AuthorityKeyIdentifier, BasicConstraints, KeyUsage, NameConstraints, SubjectKeyIdentifier,
    },
  },
};

#[cfg(feature = "ccadb")]
pub(crate) type CvTrustAnchorRaw = (
  Option<[u8; 20]>,
  bool,
  bool,
  Option<(u8, u8)>,
  &'static [u8],
  Option<([u8; 20], bool)>,
  (&'static [u8], Option<(&'static [u8], u8)>, &'static [u8]),
  (i64, i64),
);

/// Chain Validation - Trust Anchor
///
/// A trust anchor is the top-most certificate in a certificate chain.
///
/// ```ignore
/// Trust Anchors <- Intermediates <- End Entity
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CvTrustAnchor<B> {
  authority_key_identifier: Option<AuthorityKeyIdentifier>,
  has_unknown_critical_extension: bool,
  is_self_signed: bool,
  key_usage: Option<KeyUsage>,
  name_constraints: Option<NameConstraints<B>>,
  subject: B,
  subject_key_identifier: Option<FlaggedExtension<SubjectKeyIdentifier>>,
  subject_public_key_info: SubjectPublicKeyInfo<B>,
  validity: Validity,
}

impl<'cert> CvTrustAnchor<&'cert [u8]> {
  /// New instances that consumes the originating certificate.
  #[inline]
  pub fn from_certificate(certificate: Certificate<&'cert [u8]>) -> crate::Result<Self> {
    let (_, _, tbs) = certificate.into_parts();
    let parts = extensions!(
      tbs,
      extension => extension.extn_value.bytes()
    );
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

impl<B0> CvTrustAnchor<B0>
where
  B0: Lease<[u8]>,
{
  /// New instance that transforms the `B0` bytes into `B1` without consuming the originating certificate.
  #[inline]
  pub fn from_certificate_ref<'cert, B1>(certificate: &'cert Certificate<B1>) -> crate::Result<Self>
  where
    B0: TryFrom<&'cert [u8]>,
    B0::Error: Into<crate::Error>,
    B1: Lease<[u8]>,
  {
    let tbs = certificate.tbs_certificate();
    let parts = extensions!(
      tbs,
      extension => extension.extn_value.bytes().lease()
    );
    let spki = &certificate.tbs_certificate().subject_public_key_info;
    let subject_public_key_info = SubjectPublicKeyInfo::new(
      AlgorithmIdentifier::new(
        spki.algorithm.algorithm,
        if let Some(elem) = &spki.algorithm.parameters {
          Some(Any::new(
            elem.bytes().lease().try_into().map_err(Into::into)?,
            elem.tag(),
            *elem.len(),
          ))
        } else {
          None
        },
      ),
      BitString::from_bytes(
        spki.subject_public_key.bytes().lease().try_into().map_err(Into::into)?,
      ),
    );
    Ok(Self {
      authority_key_identifier: parts.authority_key_identifier,
      has_unknown_critical_extension: parts.has_unknown_critical_extension,
      is_self_signed: parts.is_self_signed,
      key_usage: parts.key_usage,
      name_constraints: parts.name_constraints,
      subject: tbs.subject.bytes().lease().try_into().map_err(Into::into)?,
      subject_key_identifier: parts.subject_key_identifier,
      subject_public_key_info,
      validity: tbs.validity.clone(),
    })
  }

  /// This constructor doesn't perform checks that assert correctness.
  #[inline]
  pub const fn new(
    authority_key_identifier: Option<AuthorityKeyIdentifier>,
    has_unknown_critical_extension: bool,
    is_self_signed: bool,
    key_usage: Option<KeyUsage>,
    name_constraints: Option<NameConstraints<B0>>,
    subject: B0,
    subject_key_identifier: Option<FlaggedExtension<SubjectKeyIdentifier>>,
    subject_public_key_info: SubjectPublicKeyInfo<B0>,
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
  pub(crate) fn _from_raw(raw: CvTrustAnchorRaw) -> crate::Result<Self>
  where
    B0: TryFrom<&'static [u8]>,
    B0::Error: Into<crate::Error>,
  {
    use crate::{
      asn1::{Len, Oid},
      calendar::DateTime,
      collections::ArrayVectorU8,
      x509::{KeyIdentifier, Time},
    };

    Ok(CvTrustAnchor::new(
      raw.0.map(|el| {
        AuthorityKeyIdentifier::new(Some(KeyIdentifier::new(ArrayVectorU8::from_array(el))))
      }),
      raw.1,
      raw.2,
      raw.3.map(KeyUsage::new),
      None,
      raw.4.try_into().map_err(Into::into)?,
      raw.5.map(|el| {
        let ki = KeyIdentifier::new(ArrayVectorU8::from_array(el.0));
        FlaggedExtension::new(SubjectKeyIdentifier::new(ki), el.1)
      }),
      {
        let algorithm = Oid::from_bytes(raw.6.0.lease())?;
        let parameters = if let Some((bytes, tag)) = raw.6.1 {
          let len = Len::from_u8(bytes.len().try_into()?);
          Some(Any::new(bytes.try_into().map_err(Into::into)?, tag, len))
        } else {
          None
        };
        SubjectPublicKeyInfo::new(
          AlgorithmIdentifier::new(algorithm, parameters),
          BitString::from_bytes(raw.6.2.try_into().map_err(Into::into)?),
        )
      },
      Validity::new(
        Time::new(DateTime::from_timestamp_secs(raw.7.0)?, false),
        Time::new(DateTime::from_timestamp_secs(raw.7.1)?, false),
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
  pub const fn name_constraints(&self) -> &Option<NameConstraints<B0>> {
    &self.name_constraints
  }

  /// Raw bytes of the subject's field
  #[inline]
  pub const fn subject(&self) -> &B0 {
    &self.subject
  }

  /// See [`SubjectKeyIdentifier`].
  #[inline]
  pub const fn subject_key_identifier(&self) -> &Option<FlaggedExtension<SubjectKeyIdentifier>> {
    &self.subject_key_identifier
  }

  /// See [`SubjectPublicKeyInfo`].
  #[inline]
  pub const fn subject_public_key_info(&self) -> &SubjectPublicKeyInfo<B0> {
    &self.subject_public_key_info
  }

  /// See [`Validity`].
  #[inline]
  pub const fn validity(&self) -> &Validity {
    &self.validity
  }
}

struct CvTrustAnchorExtensions<B> {
  authority_key_identifier: Option<AuthorityKeyIdentifier>,
  has_unknown_critical_extension: bool,
  is_self_signed: bool,
  key_usage: Option<KeyUsage>,
  name_constraints: Option<NameConstraints<B>>,
  subject_key_identifier: Option<FlaggedExtension<SubjectKeyIdentifier>>,
}
