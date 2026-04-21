use crate::{
  asn1::{Asn1DecodeWrapper, OID_X509_EXT_CRL_NUMBER, OID_X509_EXT_INVALIDITY_DATE},
  codec::{Decode, DecodeWrapper},
  x509::{CrlReason, Extensions, RevokedCertificate, SerialNumber, Time},
};

/// Chain Validation - Revoked Certificate
#[derive(Debug, PartialEq)]
pub struct CvRevokedCertificate {
  invalidity_date: Option<Time>,
  reason_code: Option<CrlReason>,
  revocation_date: Time,
  serial_number: SerialNumber,
}

impl<'bytes> TryFrom<RevokedCertificate<'bytes>> for CvRevokedCertificate {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: RevokedCertificate<'bytes>) -> Result<Self, Self::Error> {
    let mut reason_code = None;
    let mut invalidity_date = None;
    extensions(value.crl_entry_extensions.as_ref(), &mut reason_code, &mut invalidity_date)?;
    Ok(Self {
      invalidity_date,
      reason_code,
      revocation_date: value.revocation_date,
      serial_number: value.user_certificate.clone(),
    })
  }
}

impl<'any, 'bytes> TryFrom<&'any RevokedCertificate<'bytes>> for CvRevokedCertificate {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &'any RevokedCertificate<'bytes>) -> Result<Self, Self::Error> {
    let mut reason_code = None;
    let mut invalidity_date = None;
    extensions(value.crl_entry_extensions.as_ref(), &mut reason_code, &mut invalidity_date)?;
    Ok(Self {
      invalidity_date,
      reason_code,
      revocation_date: value.revocation_date,
      serial_number: value.user_certificate.clone(),
    })
  }
}

fn extensions<'bytes>(
  extensions_opt: Option<&Extensions<'bytes>>,
  reason_code: &mut Option<CrlReason>,
  invalidity_date: &mut Option<Time>,
) -> crate::Result<()> {
  if let Some(extensions) = extensions_opt {
    for extension in &extensions.entries {
      let decode_aux = Asn1DecodeWrapper::default();
      let mut dw = DecodeWrapper::new(extension.extn_value.bytes(), decode_aux);
      match extension.extn_id {
        el if el == OID_X509_EXT_CRL_NUMBER => {
          *reason_code = Some(CrlReason::decode(&mut dw)?);
        }
        el if el == OID_X509_EXT_INVALIDITY_DATE => {
          *invalidity_date = Some(Time::decode(&mut dw)?);
        }
        _ => {}
      }
    }
  }
  Ok(())
}
