#[cfg(feature = "rustls-webpki")]
mod rustls_webpki;

use crate::tls::SignatureScheme;
use core::time::Duration;

/// Verifies client certificates
pub trait ClientCertVerifier {
  /// Verifies if `end_cert` is valid to at least one of the trust anchors.
  fn verify_client_end_cert(
    &self,
    end_cert: &[u8],
    intermediates: &[&[u8]; 4],
    now: Duration,
  ) -> crate::Result<()>;

  /// Verifies if a message was really signed by teh given signer.
  fn verify_client_signature(
    &self,
    cert: &[u8],
    signer: (SignatureScheme, &[u8]),
    msg: &[u8],
  ) -> crate::Result<()>;
}
