/// The top-most certificate in a certificate chain. At the end of the day this certificate
/// leads to the root CA, regardless of the amount of intermediates.
///
/// Allows the verification of certificates using the minimum amount of associated elements.
#[derive(Debug)]
pub struct TrustAnchor<'any> {
  /// The entity this certificate identifies.
  pub subject: &'any [u8],
  /// See [`crate::x509::SubjectPublicKeyInfo`]
  pub subject_public_key_info: &'any [u8],
  /// Name constraints of the trust anchor, if any.
  pub name_constraints: Option<&'any [u8]>,
}
