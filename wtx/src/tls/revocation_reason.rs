/// Identifies the reason for the certificate revocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RevocationReasonCode {
  /// Unspecified
  Unspecified = 0,
  /// KeyCompromise
  KeyCompromise = 1,
  /// CaCompromise
  CaCompromise = 2,
  /// AffiliationChanged
  AffiliationChanged = 3,
  /// Superseded
  Superseded = 4,
  /// CessationOfOperation
  CessationOfOperation = 5,
  /// CertificateHold
  CertificateHold = 6,
  /// RemoveFromCrl
  RemoveFromCrl = 8,
  /// PrivilegeWithdrawn
  PrivilegeWithdrawn = 9,
  /// AaCompromise
  AaCompromise = 10,
}
