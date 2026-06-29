macro_rules! extensions {
  ($tbs:expr, $ext:ident => $bytes_expr:expr) => {{
    let mut crl_number = None;
    let mut issuing_distribution_point = None;

    if let Some(extensions) = $tbs.crl_extensions.as_ref() {
      for $ext in &extensions.entries {
        let decode_aux = Asn1DecodeWrapperAux::default();
        let mut dw = DecodeWrapper::new($bytes_expr, decode_aux);
        match $ext.extn_id {
          el if el == OID_X509_EXT_CRL_NUMBER => {
            let elem = CrlNumber::decode(&mut dw)?;
            if $ext.critical {
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

    CvCrlExtensions { issuing_distribution_point }
  }};
}

use crate::{
  asn1::{
    Asn1DecodeWrapperAux, OID_X509_EXT_CRL_NUMBER, OID_X509_EXT_ISSUER_DISTRIBUTION_POINT,
    Octetstring,
  },
  codec::{Decode as _, DecodeWrapper},
  collections::Vector,
  misc::Lease,
  x509::{
    Crl, Extension, Extensions, RevokedCertificate, RevokedCertificates, Time, X509CvError,
    extensions::{CrlNumber, IssuingDistributionPoint},
  },
};

/// Chain Validation - CRL
///
/// Full X.509 certificates are huge and because of that, this particular structure only has the
/// fields required to perform a chain validation.
#[derive(Clone, Debug, PartialEq)]
pub struct CvCrl<B> {
  pub(crate) issuer: B,
  pub(crate) issuing_distribution_point: Option<IssuingDistributionPoint<B>>,
  pub(crate) next_update: Option<Time>,
  pub(crate) revoked_certs: Option<RevokedCertificates<B>>,
}

impl<'cert, B0, B1> TryFrom<&'cert Crl<B0>> for CvCrl<B1>
where
  B0: Lease<[u8]>,
  B1: Lease<[u8]> + TryFrom<&'cert [u8]>,
  B1::Error: Into<crate::Error>,
{
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &'cert Crl<B0>) -> Result<Self, Self::Error> {
    let extensions = extensions!(
      &value.tbs_cert_list,
      extension => extension.extn_value.bytes().lease()
    );
    Ok(Self {
      issuer: value.tbs_cert_list.issuer.bytes().lease().try_into().map_err(Into::into)?,
      issuing_distribution_point: extensions.issuing_distribution_point,
      next_update: value.tbs_cert_list.next_update,
      revoked_certs: {
        if let Some(revoked_certificates) = &value.tbs_cert_list.revoked_certificates {
          let mut rslt = Vector::new();
          for revoked_certificate in &revoked_certificates.0 {
            rslt.push(RevokedCertificate {
              user_certificate: revoked_certificate.user_certificate.clone(),
              revocation_date: revoked_certificate.revocation_date,
              crl_entry_extensions: if let Some(exts) = &revoked_certificate.crl_entry_extensions {
                let mut entries = Vector::new();
                for ext in &exts.entries {
                  entries.push(Extension {
                    extn_id: ext.extn_id,
                    critical: ext.critical,
                    extn_value: Octetstring::from_bytes(
                      ext.extn_value.bytes().lease().try_into().map_err(Into::into)?,
                    ),
                  })?;
                }
                Some(Extensions { entries, tag: exts.tag })
              } else {
                None
              },
            })?;
          }
          Some(RevokedCertificates(rslt))
        } else {
          None
        }
      },
    })
  }
}

impl<'cert> TryFrom<Crl<&'cert [u8]>> for CvCrl<&'cert [u8]> {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: Crl<&'cert [u8]>) -> Result<Self, Self::Error> {
    let extensions = extensions!(
      &value.tbs_cert_list,
      extension => extension.extn_value.bytes()
    );
    Ok(Self {
      issuer: value.tbs_cert_list.issuer.bytes(),
      issuing_distribution_point: extensions.issuing_distribution_point,
      next_update: value.tbs_cert_list.next_update,
      revoked_certs: value.tbs_cert_list.revoked_certificates,
    })
  }
}

struct CvCrlExtensions<B> {
  issuing_distribution_point: Option<IssuingDistributionPoint<B>>,
}
