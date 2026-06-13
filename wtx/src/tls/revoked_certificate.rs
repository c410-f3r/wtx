use crate::tls::{RevocationReasonCode, SerialNumber};
use core::time::Duration;

/// A certificate that was revoked before its expiration.
pub struct RevokedCertificate {
  /// Revocation date
  pub revoked_at: Duration,
  /// See [RevocationReasonCode].
  pub rrc: Option<RevocationReasonCode>,
  /// See [SerialNumber]
  pub serial_number: SerialNumber,
  /// Suspicious date where the private key was compromised
  pub suspition_date: Option<Duration>,
}
