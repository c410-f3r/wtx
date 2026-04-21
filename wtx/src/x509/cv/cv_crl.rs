use crate::{
  asn1::{Asn1DecodeWrapper, OID_X509_EXT_CRL_NUMBER, OID_X509_EXT_ISSUER_DISTRIBUTION_POINT},
  codec::{Decode, DecodeWrapper},
  misc::RefOrOwned,
  x509::{
    Crl, NameVector, RevokedCertificates, TbsCertList, Time, X509CvError,
    extensions::{CrlNumber, IssuingDistributionPoint},
  },
};

/// Chain Validation - CRL
///
/// Full X.509 certificates are huge and because of that, this particular structure only has the
/// fields required to perform a chain validation.
#[derive(Debug, PartialEq)]
pub struct CvCrl<'any, 'bytes> {
  pub(crate) issuer: RefOrOwned<'any, NameVector<'bytes>>,
  pub(crate) issuing_distribution_point: Option<IssuingDistributionPoint<'bytes>>,
  pub(crate) next_update: Option<Time>,
  pub(crate) revoked_certs: RefOrOwned<'any, Option<RevokedCertificates<'bytes>>>,
}

impl<'any, 'bytes> TryFrom<Crl<'bytes>> for CvCrl<'any, 'bytes> {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: Crl<'bytes>) -> Result<Self, Self::Error> {
    let parts = Parts::new(&value.tbs_cert_list)?;
    Ok(Self {
      issuer: RefOrOwned::Right(value.tbs_cert_list.issuer),
      issuing_distribution_point: parts.issuing_distribution_point,
      next_update: value.tbs_cert_list.next_update,
      revoked_certs: RefOrOwned::Right(value.tbs_cert_list.revoked_certificates),
    })
  }
}

impl<'any, 'bytes> TryFrom<&'any Crl<'bytes>> for CvCrl<'any, 'bytes> {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &'any Crl<'bytes>) -> Result<Self, Self::Error> {
    let parts = Parts::new(&value.tbs_cert_list)?;
    Ok(Self {
      issuer: RefOrOwned::Left(&value.tbs_cert_list.issuer),
      issuing_distribution_point: parts.issuing_distribution_point,
      next_update: value.tbs_cert_list.next_update,
      revoked_certs: RefOrOwned::Left(&value.tbs_cert_list.revoked_certificates),
    })
  }
}

struct Parts<'bytes> {
  issuing_distribution_point: Option<IssuingDistributionPoint<'bytes>>,
}

impl<'bytes> Parts<'bytes> {
  fn new(tbs: &TbsCertList<'bytes>) -> crate::Result<Self> {
    let mut crl_number = None;
    let mut issuing_distribution_point = None;

    if let Some(extensions) = tbs.crl_extensions.as_ref() {
      for extension in &extensions.entries {
        let decode_aux = Asn1DecodeWrapper::default();
        let mut dw = DecodeWrapper::new(extension.extn_value.bytes(), decode_aux);
        match extension.extn_id {
          el if el == OID_X509_EXT_CRL_NUMBER => {
            let elem = CrlNumber::decode(&mut dw)?;
            if extension.critical {
              return Err(X509CvError::CrlNumberMustNotBeCritical.into());
            }
            crl_number = Some(elem);
          }
          el if el == OID_X509_EXT_ISSUER_DISTRIBUTION_POINT => {
            issuing_distribution_point = Some(IssuingDistributionPoint::decode(&mut dw)?);
          }
          _ => {}
        }
      }
    }

    if crl_number.is_none() {
      return Err(X509CvError::MissingCrlNumber.into());
    }

    Ok(Self { issuing_distribution_point })
  }
}
