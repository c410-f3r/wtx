use crate::{
  asn1::{Integer, Time},
  collection::Vector,
  x509::Extension,
};

/// A revoked certificate entry in a Certificate Revocation List (CRL)
#[derive(Debug, PartialEq)]
pub struct RevokedCertificate<'bytes> {
  /// Additional information.
  pub crl_entry_extensions: Option<Vector<Extension<'bytes>>>,
  /// The date and time when the certificate was revoked.
  pub revocation_date: Time,
  /// Serial number of the revoked certificate.
  pub user_certificate: Integer<&'bytes [u8]>,
}
