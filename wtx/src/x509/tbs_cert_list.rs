use crate::{
  asn1::Time,
  collection::Vector,
  x509::{AlgorithmIdentifier, Extension, Name, RevokedCertificate},
};

/// A sequence containing the name of the issuer, issue date, issue date of the next list, the
/// optional list of revoked certificates, and optional CRL extensions.
#[derive(Debug, PartialEq)]
pub struct TbsCertList<'bytes> {
  /// Additional information.
  pub crl_extensions: Option<Vector<Extension<'bytes>>>,
  /// The issuer name identifies the entity that has signed and issued the CRL.
  pub issuer: Name<'bytes>,
  /// The date by which the next CRL will be issued.
  pub next_update: Option<Time>,
  /// See [`RevokedCertificate`].
  pub revoked_certificates: Option<Vector<RevokedCertificate<'bytes>>>,
  /// See [`AlgorithmIdentifier`].
  pub signature: AlgorithmIdentifier<'bytes>,
  /// Indicates the issue date of this CRL.
  pub this_update: Time,
}
