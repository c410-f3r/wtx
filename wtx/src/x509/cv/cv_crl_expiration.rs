/// Chain Validation - Crl Expiration
///
/// Controls how the CRL `nextUpdate` expiration field is interpreted when verifying chains.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CvCrlExpiration {
  /// Raises an error if the CRL is expired.
  ///
  /// This variant is often more secure.
  #[default]
  Enforce,
  /// Expired CRLs are ignored.
  Ignore,
}
