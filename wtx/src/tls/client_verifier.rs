//#[cfg(feature = "rustls-webpki")]
//mod rustls_webpki;

use crate::{collection::ArrayVectorU8, tls::protocol::signature_scheme::SignatureScheme};
use core::time::Duration;

/// Verifies client certificates
pub trait ClientVerifier {
  /// Verifies if `end_cert` is valid to at least one of the trust anchors.
  fn verify_identity(
    &self,
    end_cert: &[u8],
    intermediates: ArrayVectorU8<&[u8], 2>,
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
