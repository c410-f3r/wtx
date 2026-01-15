use crate::tls::SignedCertificateData;
use core::time::Duration;

#[derive(Debug)]
pub struct CertificateRevocationList<'any> {
  issuer: &'any [u8],
  issuing_distribution_point: Option<&'any [u8]>,
  next_update: Duration,
  revoked_certs: &'any [u8],
  signed_data: SignedCertificateData<'any>,
}
