/// A root CA with the minimum required information that allows the verification of certificates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrustAnchor<'any> {
  /// Value of the `subject` field.
  pub subject: &'any [u8],
  /// Value of the `subjectPublicKeyInfo` field.
  pub subject_public_key_info: &'any [u8],
  /// Value of the `NameConstraints` field.
  pub name_constraints: Option<&'any [u8]>,
}

#[cfg(feature = "rustls-webpki")]
impl<'any> From<&TrustAnchor<'any>> for rustls_pki_types::TrustAnchor<'any> {
  fn from(value: &TrustAnchor<'any>) -> Self {
    Self {
      subject: rustls_pki_types::Der::from_slice(value.subject),
      subject_public_key_info: rustls_pki_types::Der::from_slice(value.subject_public_key_info),
      name_constraints: value.name_constraints.map(rustls_pki_types::Der::from_slice),
    }
  }
}
