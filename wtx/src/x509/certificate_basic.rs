use crate::{
  asn1::{
    Asn1DecodeWrapper, OID_CT_LIST_SCT, OID_X509_EXT_BASIC_CONSTRAINTS,
    OID_X509_EXT_CRL_DISTRIBUTION_POINTS, OID_X509_EXT_EXTENDED_KEY_USAGE, OID_X509_EXT_KEY_USAGE,
    OID_X509_EXT_NAME_CONSTRAINTS, OID_X509_EXT_SUBJECT_ALT_NAME,
  },
  codec::{Decode, GenericDecodeWrapper},
  misc::{Lease, RefOrOwned},
  x509::{
    AlgorithmIdentifier, Certificate, Extensions, Name, SerialNumber, SubjectPublicKeyInfo,
    Validity,
    extensions::{
      BasicConstraints, CrlDistributionPoints, ExtendedKeyUsage, KeyUsage, NameConstraints,
      SignedCertificateTimestampList, SubjectAlternativeName,
    },
  },
};

/// Full X.509 certificates are huge and because of that, this particular structure only has the fields
/// required to perform a chain validation.
#[derive(Debug, PartialEq)]
pub struct CertificateBasic<'any, 'bytes> {
  pub(crate) basic_constraints: Option<BasicConstraints>,
  pub(crate) crl_distribution_points: Option<CrlDistributionPoints<'bytes>>,
  pub(crate) eku: Option<ExtendedKeyUsage>,
  pub(crate) issuer: RefOrOwned<'any, Name<'bytes>>,
  pub(crate) key_usage: Option<KeyUsage>,
  pub(crate) name_constraints: Option<NameConstraints<'bytes>>,
  pub(crate) scts: Option<SignedCertificateTimestampList<'bytes>>,
  pub(crate) serial: SerialNumber,
  pub(crate) signature_algorithm: RefOrOwned<'any, AlgorithmIdentifier<'bytes>>,
  pub(crate) signature_msg: &'bytes [u8],
  pub(crate) signature: &'bytes [u8],
  pub(crate) spki: RefOrOwned<'any, SubjectPublicKeyInfo<'bytes>>,
  pub(crate) subject_alt_name: Option<SubjectAlternativeName<'bytes>>,
  pub(crate) subject: RefOrOwned<'any, Name<'bytes>>,
  pub(crate) validity: Validity,
}

impl<'any, 'bytes> Lease<CertificateBasic<'any, 'bytes>> for CertificateBasic<'any, 'bytes> {
  #[inline]
  fn lease(&self) -> &CertificateBasic<'any, 'bytes> {
    self
  }
}

impl<'any, 'bytes> TryFrom<Certificate<'bytes>> for CertificateBasic<'any, 'bytes> {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: Certificate<'bytes>) -> Result<Self, Self::Error> {
    let mut basic_constraints = None;
    let mut crl_distribution_points = None;
    let mut eku = None;
    let mut key_usage = None;
    let mut name_constraints = None;
    let mut scts = None;
    let mut subject_alt_name = None;
    extensions(
      &mut basic_constraints,
      &mut crl_distribution_points,
      &mut eku,
      value.tbs_certificate.extensions.as_ref(),
      &mut key_usage,
      &mut name_constraints,
      &mut scts,
      &mut subject_alt_name,
    )?;
    Ok(Self {
      basic_constraints,
      crl_distribution_points,
      eku,
      issuer: RefOrOwned::Right(value.tbs_certificate.issuer),
      key_usage,
      name_constraints,
      scts,
      serial: value.tbs_certificate.serial_number.clone(),
      signature_algorithm: RefOrOwned::Right(value.signature_algorithm),
      signature_msg: value.tbs_certificate.bytes,
      signature: value.signature_value.bytes(),
      spki: RefOrOwned::Right(value.tbs_certificate.subject_public_key_info),
      subject_alt_name,
      subject: RefOrOwned::Right(value.tbs_certificate.subject),
      validity: value.tbs_certificate.validity.clone(),
    })
  }
}

impl<'any, 'bytes> TryFrom<&'any Certificate<'bytes>> for CertificateBasic<'any, 'bytes> {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &'any Certificate<'bytes>) -> Result<Self, Self::Error> {
    let mut basic_constraints = None;
    let mut crl_distribution_points = None;
    let mut eku = None;
    let mut key_usage = None;
    let mut name_constraints = None;
    let mut scts = None;
    let mut subject_alt_name = None;
    extensions(
      &mut basic_constraints,
      &mut crl_distribution_points,
      &mut eku,
      value.tbs_certificate.extensions.as_ref(),
      &mut key_usage,
      &mut name_constraints,
      &mut scts,
      &mut subject_alt_name,
    )?;
    Ok(Self {
      basic_constraints,
      crl_distribution_points,
      eku,
      issuer: RefOrOwned::Left(&value.tbs_certificate.issuer),
      key_usage,
      name_constraints,
      scts,
      serial: value.tbs_certificate.serial_number.clone(),
      signature_algorithm: RefOrOwned::Left(&value.signature_algorithm),
      signature_msg: value.tbs_certificate.bytes,
      signature: value.signature_value.bytes(),
      spki: RefOrOwned::Left(&value.tbs_certificate.subject_public_key_info),
      subject_alt_name,
      subject: RefOrOwned::Left(&value.tbs_certificate.subject),
      validity: value.tbs_certificate.validity.clone(),
    })
  }
}

fn extensions<'bytes, const N: u8>(
  basic_constraints: &mut Option<BasicConstraints>,
  crl_distribution_points: &mut Option<CrlDistributionPoints<'bytes>>,
  eku: &mut Option<ExtendedKeyUsage>,
  extensions_opt: Option<&Extensions<'bytes, N>>,
  key_usage: &mut Option<KeyUsage>,
  name_constraints: &mut Option<NameConstraints<'bytes>>,
  scts: &mut Option<SignedCertificateTimestampList<'bytes>>,
  subject_alt_name: &mut Option<SubjectAlternativeName<'bytes>>,
) -> crate::Result<()> {
  if let Some(extensions) = extensions_opt {
    for ext in extensions.0.iter() {
      let decode_aux = Asn1DecodeWrapper::default();
      let mut dw = GenericDecodeWrapper::new(ext.extn_value.bytes(), decode_aux);
      match ext.extn_id {
        el if el == OID_X509_EXT_BASIC_CONSTRAINTS => {
          *basic_constraints = Some(BasicConstraints::decode(&mut dw)?);
        }
        el if el == OID_X509_EXT_CRL_DISTRIBUTION_POINTS => {
          *crl_distribution_points = Some(CrlDistributionPoints::decode(&mut dw)?);
        }
        el if el == OID_X509_EXT_EXTENDED_KEY_USAGE => {
          *eku = Some(ExtendedKeyUsage::decode(&mut dw)?);
        }
        el if el == OID_X509_EXT_KEY_USAGE => {
          *key_usage = Some(KeyUsage::decode(&mut dw)?);
        }
        el if el == OID_X509_EXT_NAME_CONSTRAINTS => {
          *name_constraints = Some(NameConstraints::decode(&mut dw)?);
        }
        el if el == OID_X509_EXT_SUBJECT_ALT_NAME => {
          *subject_alt_name = Some(SubjectAlternativeName::decode(&mut dw)?);
        }
        el if el == OID_CT_LIST_SCT => {
          *scts = Some(SignedCertificateTimestampList::decode(&mut dw)?);
        }
        _ => {}
      }
    }
  }
  Ok(())
}
