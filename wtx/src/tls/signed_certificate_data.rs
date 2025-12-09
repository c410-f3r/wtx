/// Data of a signed certificate
#[derive(Debug)]
pub struct SignedCertificateData<'any> {
  algorithm: &'any [u8],
  data: &'any [u8],
  signature: &'any [u8],
}
